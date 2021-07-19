use std::cell::RefCell;
use std::rc::{Rc, Weak};

use actix_router::ResourceDef;
use ahash::AHashMap;
use url::Url;

use crate::error::UrlGenerationError;
use crate::request::HttpRequest;

#[derive(Clone, Debug)]
pub struct ResourceMap {
    pattern: ResourceDef,

    /// Named resources within the tree or, for external resources,
    /// it points to isolated nodes outside the tree.
    named: AHashMap<String, Rc<ResourceMap>>,

    parent: RefCell<Weak<ResourceMap>>,

    /// Must be `None` for "terminating" patterns
    nodes: Option<Vec<Rc<ResourceMap>>>,
}

impl ResourceMap {
    /// Creates a _container_ node in the `ResourceMap` tree.
    pub fn new(root: ResourceDef) -> Self {
        ResourceMap {
            pattern: root,
            named: AHashMap::default(),
            parent: RefCell::new(Weak::new()),
            nodes: Some(Vec::new()),
        }
    }

    /// Adds a (possibly nested) resource.
    ///
    /// To add a terminating pattern, `nested` must be `None`.
    /// To add external resource, supply a pattern without a leading `/`.
    /// The root pattern of `nested`, if present, should match `pattern`.
    pub fn add(&mut self, pattern: &mut ResourceDef, nested: Option<Rc<ResourceMap>>) {
        pattern.set_id(self.nodes.as_ref().unwrap().len() as u16);

        if let Some(new_node) = nested {
            assert_eq!(new_node.pattern.pattern(), pattern.pattern());
            self.named.extend(new_node.named.clone().into_iter());
            self.nodes.as_mut().unwrap().push(new_node);
        } else {
            let new_node = Rc::new(ResourceMap {
                pattern: pattern.clone(),
                named: AHashMap::default(),
                parent: RefCell::new(Weak::new()),
                nodes: None,
            });

            if !pattern.name().is_empty() {
                self.named
                    .insert(pattern.name().to_owned(), Rc::clone(&new_node));
            }

            // Don't add external resources to the tree
            if pattern.pattern().is_empty() || pattern.pattern().starts_with('/') {
                self.nodes.as_mut().unwrap().push(new_node);
            }
        }
    }

    pub(crate) fn finish(&self, this: Rc<ResourceMap>) {
        for node in this.nodes.iter().flatten() {
            *node.parent.borrow_mut() = Rc::downgrade(&this);
            node.finish(Rc::clone(node));
        }
    }

    /// Generate url for named resource
    ///
    /// Check [`HttpRequest::url_for()`](../struct.HttpRequest.html#method.
    /// url_for) for detailed information.
    pub fn url_for<U, I>(
        &self,
        req: &HttpRequest,
        name: &str,
        elements: U,
    ) -> Result<Url, UrlGenerationError>
    where
        U: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        let mut elements = elements.into_iter();

        let path = self
            .named
            .get(name)
            .ok_or(UrlGenerationError::ResourceNotFound)?
            .fold_parents(String::new(), |mut acc, node| {
                if node.pattern.resource_path(&mut acc, &mut elements) {
                    Some(acc)
                } else {
                    None
                }
            })
            .ok_or(UrlGenerationError::NotEnoughElements)?;

        if path.starts_with('/') {
            let conn = req.connection_info();
            Ok(Url::parse(&format!(
                "{}://{}{}",
                conn.scheme(),
                conn.host(),
                path
            ))?)
        } else {
            Ok(Url::parse(&path)?)
        }
    }

    pub fn has_resource(&self, path: &str) -> bool {
        self.find_matching_node(path).is_some()
    }

    /// Returns the name of the route that matches the given path or None if no full match
    /// is possible or the matching resource is not named.
    pub fn match_name(&self, path: &str) -> Option<&str> {
        self.find_matching_node(path)
            .and_then(|node| match node.pattern.name() {
                "" => None,
                s => Some(s),
            })
    }

    /// Returns the full resource pattern matched against a path or None if no full match
    /// is possible.
    pub fn match_pattern(&self, path: &str) -> Option<String> {
        self.find_matching_node(path)?
            .fold_parents(String::new(), |mut acc, node| {
                acc.push_str(node.pattern.pattern());
                Some(acc)
            })
    }

    fn find_matching_node(&self, path: &str) -> Option<&ResourceMap> {
        self._find_matching_node(path).flatten()
    }

    /// Returns `None` if root pattern doesn't match;
    /// `Some(None)` if root pattern matches but there is no matching child pattern.
    /// Don't search sideways when `Some(none)` is returned.
    fn _find_matching_node(&self, path: &str) -> Option<Option<&ResourceMap>> {
        let matched_len = if path.is_empty() && self.pattern.pattern().is_empty() {
            // ResourceDef::is_prefix_match has a bug where empty pattern doesn't match empty path
            0
        } else {
            self.pattern.is_prefix_match(path)?
        };
        let path = &path[matched_len..];

        Some(match &self.nodes {
            Some(nodes) => nodes
                .iter()
                .filter_map(|node| node._find_matching_node(path))
                .next()
                .flatten(),

            None => Some(self),
        })
    }

    /// Folds the parents from the root of the tree to self.
    fn fold_parents<F, B>(&self, init: B, mut f: F) -> Option<B>
    where
        F: FnMut(B, &ResourceMap) -> Option<B>,
    {
        self._fold_parents(init, &mut f)
    }

    fn _fold_parents<F, B>(&self, init: B, f: &mut F) -> Option<B>
    where
        F: FnMut(B, &ResourceMap) -> Option<B>,
    {
        let data = match self.parent.borrow().upgrade() {
            Some(ref parent) => parent._fold_parents(init, f)?,
            None => init,
        };

        f(data, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_matched_pattern() {
        let mut root = ResourceMap::new(ResourceDef::root_prefix(""));

        let mut user_map = ResourceMap::new(ResourceDef::root_prefix("/user/{id}"));
        user_map.add(&mut ResourceDef::new("/"), None);
        user_map.add(&mut ResourceDef::new("/profile"), None);
        user_map.add(&mut ResourceDef::new("/article/{id}"), None);
        user_map.add(&mut ResourceDef::new("/post/{post_id}"), None);
        user_map.add(
            &mut ResourceDef::new("/post/{post_id}/comment/{comment_id}"),
            None,
        );

        root.add(&mut ResourceDef::new("/info"), None);
        root.add(&mut ResourceDef::new("/v{version:[[:digit:]]{1}}"), None);
        root.add(
            &mut ResourceDef::root_prefix("/user/{id}"),
            Some(Rc::new(user_map)),
        );
        root.add(&mut ResourceDef::new("/info"), None);

        let root = Rc::new(root);
        root.finish(Rc::clone(&root));

        // sanity check resource map setup

        assert!(root.has_resource("/info"));
        assert!(!root.has_resource("/bar"));

        assert!(root.has_resource("/v1"));
        assert!(root.has_resource("/v2"));
        assert!(!root.has_resource("/v33"));

        assert!(root.has_resource("/user/22"));
        assert!(root.has_resource("/user/22/"));
        assert!(root.has_resource("/user/22/profile"));

        // extract patterns from paths

        assert!(root.match_pattern("/bar").is_none());
        assert!(root.match_pattern("/v44").is_none());

        assert_eq!(root.match_pattern("/info"), Some("/info".to_owned()));
        assert_eq!(
            root.match_pattern("/v1"),
            Some("/v{version:[[:digit:]]{1}}".to_owned())
        );
        assert_eq!(
            root.match_pattern("/v2"),
            Some("/v{version:[[:digit:]]{1}}".to_owned())
        );
        assert_eq!(
            root.match_pattern("/user/22/profile"),
            Some("/user/{id}/profile".to_owned())
        );
        assert_eq!(
            root.match_pattern("/user/602CFB82-7709-4B17-ADCF-4C347B6F2203/profile"),
            Some("/user/{id}/profile".to_owned())
        );
        assert_eq!(
            root.match_pattern("/user/22/article/44"),
            Some("/user/{id}/article/{id}".to_owned())
        );
        assert_eq!(
            root.match_pattern("/user/22/post/my-post"),
            Some("/user/{id}/post/{post_id}".to_owned())
        );
        assert_eq!(
            root.match_pattern("/user/22/post/other-post/comment/42"),
            Some("/user/{id}/post/{post_id}/comment/{comment_id}".to_owned())
        );
    }

    #[test]
    fn extract_matched_name() {
        let mut root = ResourceMap::new(ResourceDef::root_prefix(""));

        let mut rdef = ResourceDef::new("/info");
        *rdef.name_mut() = "root_info".to_owned();
        root.add(&mut rdef, None);

        let mut user_map = ResourceMap::new(ResourceDef::root_prefix("/user/{id}"));
        let mut rdef = ResourceDef::new("/");
        user_map.add(&mut rdef, None);

        let mut rdef = ResourceDef::new("/post/{post_id}");
        *rdef.name_mut() = "user_post".to_owned();
        user_map.add(&mut rdef, None);

        root.add(
            &mut ResourceDef::root_prefix("/user/{id}"),
            Some(Rc::new(user_map)),
        );

        let root = Rc::new(root);
        root.finish(Rc::clone(&root));

        // sanity check resource map setup

        assert!(root.has_resource("/info"));
        assert!(!root.has_resource("/bar"));

        assert!(root.has_resource("/user/22"));
        assert!(root.has_resource("/user/22/"));
        assert!(root.has_resource("/user/22/post/55"));

        // extract patterns from paths

        assert!(root.match_name("/bar").is_none());
        assert!(root.match_name("/v44").is_none());

        assert_eq!(root.match_name("/info"), Some("root_info"));
        assert_eq!(root.match_name("/user/22"), None);
        assert_eq!(root.match_name("/user/22/"), None);
        assert_eq!(root.match_name("/user/22/post/55"), Some("user_post"));
    }

    #[test]
    fn bug_fix_issue_1582_debug_print_exits() {
        // ref: https://github.com/actix/actix-web/issues/1582
        let mut root = ResourceMap::new(ResourceDef::root_prefix(""));

        let mut user_map = ResourceMap::new(ResourceDef::root_prefix("/user/{id}"));
        user_map.add(&mut ResourceDef::new("/"), None);
        user_map.add(&mut ResourceDef::new("/profile"), None);
        user_map.add(&mut ResourceDef::new("/article/{id}"), None);
        user_map.add(&mut ResourceDef::new("/post/{post_id}"), None);
        user_map.add(
            &mut ResourceDef::new("/post/{post_id}/comment/{comment_id}"),
            None,
        );

        root.add(
            &mut ResourceDef::root_prefix("/user/{id}"),
            Some(Rc::new(user_map)),
        );

        let root = Rc::new(root);
        root.finish(Rc::clone(&root));

        // check root has no parent
        assert!(root.parent.borrow().upgrade().is_none());
        // check child has parent reference
        assert!(root.nodes.as_ref().unwrap()[0]
            .parent
            .borrow()
            .upgrade()
            .is_some());
        // check child's parent root id matches root's root id
        assert!(Rc::ptr_eq(
            &root.nodes.as_ref().unwrap()[0]
                .parent
                .borrow()
                .upgrade()
                .unwrap(),
            &root
        ));

        let output = format!("{:?}", root);
        assert!(output.starts_with("ResourceMap {"));
        assert!(output.ends_with(" }"));
    }

    #[test]
    fn short_circuit() {
        let mut root = ResourceMap::new(ResourceDef::root_prefix(""));

        let mut user_root = ResourceDef::root_prefix("/user");
        let mut user_map = ResourceMap::new(user_root.clone());
        user_map.add(&mut ResourceDef::new("/u1"), None);
        user_map.add(&mut ResourceDef::new("/u2"), None);

        root.add(&mut ResourceDef::new("/user/u3"), None);
        root.add(&mut user_root, Some(Rc::new(user_map)));
        root.add(&mut ResourceDef::new("/user/u4"), None);

        let rmap = Rc::new(root);
        rmap.finish(Rc::clone(&rmap));

        assert!(rmap.has_resource("/user/u1"));
        assert!(rmap.has_resource("/user/u2"));
        assert!(rmap.has_resource("/user/u3"));
        assert!(!rmap.has_resource("/user/u4"));
    }
}
