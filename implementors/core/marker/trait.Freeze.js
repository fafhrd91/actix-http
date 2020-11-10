(function() {var implementors = {};
implementors["actix_files"] = [{"text":"impl Freeze for Directory","synthetic":true,"types":[]},{"text":"impl Freeze for Files","synthetic":true,"types":[]},{"text":"impl Freeze for NamedFile","synthetic":true,"types":[]},{"text":"impl Freeze for HttpRange","synthetic":true,"types":[]},{"text":"impl Freeze for FilesService","synthetic":true,"types":[]}];
implementors["actix_http"] = [{"text":"impl&lt;T, S, X, U&gt; Freeze for HttpServiceBuilder&lt;T, S, X, U&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;U: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;X: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for ServiceConfig","synthetic":true,"types":[]},{"text":"impl Freeze for Extensions","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for Message&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl !Freeze for RequestHead","synthetic":true,"types":[]},{"text":"impl !Freeze for ResponseHead","synthetic":true,"types":[]},{"text":"impl&lt;P&gt; Freeze for Request&lt;P&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;P: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;B&nbsp;=&nbsp;Body&gt; !Freeze for Response&lt;B&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for ResponseBuilder","synthetic":true,"types":[]},{"text":"impl&lt;T, S, B, X, U&gt; Freeze for HttpService&lt;T, S, B, X, U&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;U: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;X: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for KeepAlive","synthetic":true,"types":[]},{"text":"impl !Freeze for RequestHeadType","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Freeze for Payload&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for Protocol","synthetic":true,"types":[]},{"text":"impl&lt;S, E&gt; Freeze for BodyStream&lt;S, E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Freeze for SizedStream&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for BodySize","synthetic":true,"types":[]},{"text":"impl&lt;B&gt; !Freeze for ResponseBody&lt;B&gt;","synthetic":true,"types":[]},{"text":"impl !Freeze for Body","synthetic":true,"types":[]},{"text":"impl&lt;T, U&gt; Freeze for Connector&lt;T, U&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl !Freeze for Connect","synthetic":true,"types":[]},{"text":"impl Freeze for ConnectError","synthetic":true,"types":[]},{"text":"impl Freeze for FreezeRequestError","synthetic":true,"types":[]},{"text":"impl Freeze for InvalidUrl","synthetic":true,"types":[]},{"text":"impl Freeze for SendRequestError","synthetic":true,"types":[]},{"text":"impl Freeze for Protocol","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Freeze for Decoder&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;B&gt; !Freeze for Encoder&lt;B&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for ExtendedValue","synthetic":true,"types":[]},{"text":"impl Freeze for ContentEncoding","synthetic":true,"types":[]},{"text":"impl Freeze for AcceptCharset","synthetic":true,"types":[]},{"text":"impl Freeze for Accept","synthetic":true,"types":[]},{"text":"impl Freeze for AcceptLanguage","synthetic":true,"types":[]},{"text":"impl Freeze for Allow","synthetic":true,"types":[]},{"text":"impl Freeze for CacheControl","synthetic":true,"types":[]},{"text":"impl Freeze for CacheDirective","synthetic":true,"types":[]},{"text":"impl Freeze for ContentDisposition","synthetic":true,"types":[]},{"text":"impl Freeze for DispositionType","synthetic":true,"types":[]},{"text":"impl Freeze for DispositionParam","synthetic":true,"types":[]},{"text":"impl Freeze for ContentLanguage","synthetic":true,"types":[]},{"text":"impl Freeze for ContentRange","synthetic":true,"types":[]},{"text":"impl Freeze for ContentRangeSpec","synthetic":true,"types":[]},{"text":"impl Freeze for ContentType","synthetic":true,"types":[]},{"text":"impl Freeze for Date","synthetic":true,"types":[]},{"text":"impl Freeze for ETag","synthetic":true,"types":[]},{"text":"impl Freeze for Expires","synthetic":true,"types":[]},{"text":"impl Freeze for IfMatch","synthetic":true,"types":[]},{"text":"impl Freeze for IfModifiedSince","synthetic":true,"types":[]},{"text":"impl Freeze for IfNoneMatch","synthetic":true,"types":[]},{"text":"impl Freeze for IfRange","synthetic":true,"types":[]},{"text":"impl Freeze for IfUnmodifiedSince","synthetic":true,"types":[]},{"text":"impl Freeze for LastModified","synthetic":true,"types":[]},{"text":"impl Freeze for HeaderMap","synthetic":true,"types":[]},{"text":"impl Freeze for ConnectionType","synthetic":true,"types":[]},{"text":"impl Freeze for Error","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; !Freeze for InternalError&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for ParseError","synthetic":true,"types":[]},{"text":"impl Freeze for PayloadError","synthetic":true,"types":[]},{"text":"impl Freeze for DispatchError","synthetic":true,"types":[]},{"text":"impl Freeze for ContentTypeError","synthetic":true,"types":[]},{"text":"impl Freeze for ClientCodec","synthetic":true,"types":[]},{"text":"impl Freeze for ClientPayloadCodec","synthetic":true,"types":[]},{"text":"impl Freeze for Codec","synthetic":true,"types":[]},{"text":"impl&lt;T, S, B, X, U&gt; !Freeze for Dispatcher&lt;T, S, B, X, U&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for ExpectHandler","synthetic":true,"types":[]},{"text":"impl Freeze for Payload","synthetic":true,"types":[]},{"text":"impl&lt;T, S, B, X, U&gt; Freeze for H1Service&lt;T, S, B, X, U&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;U: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;X: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, S, B, X, U&gt; Freeze for H1ServiceHandler&lt;T, S, B, X, U&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for OneRequest&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for UpgradeHandler&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T, B&gt; !Freeze for SendResponse&lt;T, B&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; !Freeze for Message&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for MessageType","synthetic":true,"types":[]},{"text":"impl&lt;T, S, B&gt; !Freeze for Dispatcher&lt;T, S, B&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T, S, B&gt; Freeze for H2Service&lt;T, S, B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for Payload","synthetic":true,"types":[]},{"text":"impl !Freeze for TestRequest","synthetic":true,"types":[]},{"text":"impl Freeze for TestBuffer","synthetic":true,"types":[]},{"text":"impl Freeze for Codec","synthetic":true,"types":[]},{"text":"impl&lt;S, T&gt; Freeze for Dispatcher&lt;S, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as Service&gt;::Error: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for Parser","synthetic":true,"types":[]},{"text":"impl Freeze for CloseReason","synthetic":true,"types":[]},{"text":"impl !Freeze for Frame","synthetic":true,"types":[]},{"text":"impl !Freeze for Item","synthetic":true,"types":[]},{"text":"impl !Freeze for Message","synthetic":true,"types":[]},{"text":"impl Freeze for CloseCode","synthetic":true,"types":[]},{"text":"impl Freeze for OpCode","synthetic":true,"types":[]},{"text":"impl Freeze for ProtocolError","synthetic":true,"types":[]},{"text":"impl Freeze for HandshakeError","synthetic":true,"types":[]}];
implementors["actix_http_test"] = [{"text":"impl Freeze for TestServer","synthetic":true,"types":[]}];
implementors["actix_multipart"] = [{"text":"impl !Freeze for Field","synthetic":true,"types":[]},{"text":"impl !Freeze for Multipart","synthetic":true,"types":[]},{"text":"impl Freeze for MultipartError","synthetic":true,"types":[]}];
implementors["actix_web"] = [{"text":"impl&lt;T, B&gt; Freeze for App&lt;T, B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for HttpRequest","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for Resource&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for Route","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for Scope&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;F, I, S, B&gt; !Freeze for HttpServer&lt;F, I, S, B&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A, B&gt; Freeze for Either&lt;A, B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;B: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for AppService","synthetic":true,"types":[]},{"text":"impl Freeze for AppConfig","synthetic":true,"types":[]},{"text":"impl Freeze for ServiceConfig","synthetic":true,"types":[]},{"text":"impl&lt;T:&nbsp;?Sized&gt; Freeze for Data&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for UrlGenerationError","synthetic":true,"types":[]},{"text":"impl Freeze for UrlencodedError","synthetic":true,"types":[]},{"text":"impl Freeze for JsonPayloadError","synthetic":true,"types":[]},{"text":"impl Freeze for PathError","synthetic":true,"types":[]},{"text":"impl Freeze for QueryPayloadError","synthetic":true,"types":[]},{"text":"impl Freeze for ReadlinesError","synthetic":true,"types":[]},{"text":"impl Freeze for AnyGuard","synthetic":true,"types":[]},{"text":"impl Freeze for AllGuard","synthetic":true,"types":[]},{"text":"impl Freeze for ConnectionInfo","synthetic":true,"types":[]},{"text":"impl Freeze for Compress","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for Condition&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for DefaultHeaders","synthetic":true,"types":[]},{"text":"impl Freeze for Logger","synthetic":true,"types":[]},{"text":"impl&lt;B&gt; Freeze for ErrorHandlers&lt;B&gt;","synthetic":true,"types":[]},{"text":"impl&lt;B&gt; !Freeze for ErrorHandlerResponse&lt;B&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for NormalizePath","synthetic":true,"types":[]},{"text":"impl Freeze for TrailingSlash","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for ReqData&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl !Freeze for ResourceMap","synthetic":true,"types":[]},{"text":"impl Freeze for ServiceRequest","synthetic":true,"types":[]},{"text":"impl&lt;B&nbsp;=&nbsp;Body&gt; !Freeze for ServiceResponse&lt;B&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for WebService","synthetic":true,"types":[]},{"text":"impl !Freeze for TestRequest","synthetic":true,"types":[]},{"text":"impl Freeze for TestServerConfig","synthetic":true,"types":[]},{"text":"impl Freeze for TestServer","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for Form&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for FormConfig","synthetic":true,"types":[]},{"text":"impl&lt;U&gt; Freeze for UrlEncoded&lt;U&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for Json&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for JsonConfig","synthetic":true,"types":[]},{"text":"impl&lt;U&gt; Freeze for JsonBody&lt;U&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for Path&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for PathConfig","synthetic":true,"types":[]},{"text":"impl Freeze for Payload","synthetic":true,"types":[]},{"text":"impl Freeze for PayloadConfig","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for Query&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for QueryConfig","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Freeze for Readlines&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T as HttpMessage&gt;::Stream: Freeze,&nbsp;</span>","synthetic":true,"types":[]}];
implementors["actix_web_actors"] = [{"text":"impl&lt;A&gt; Freeze for HttpContext&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; Freeze for WebsocketContext&lt;A&gt;","synthetic":true,"types":[]}];
implementors["awc"] = [{"text":"impl !Freeze for ClientBuilder","synthetic":true,"types":[]},{"text":"impl Freeze for BoxedSocket","synthetic":true,"types":[]},{"text":"impl Freeze for FrozenClientRequest","synthetic":true,"types":[]},{"text":"impl Freeze for FrozenSendBuilder","synthetic":true,"types":[]},{"text":"impl !Freeze for ClientRequest","synthetic":true,"types":[]},{"text":"impl&lt;S&nbsp;=&nbsp;Pin&lt;Box&lt;dyn Stream&lt;Item = Result&lt;Bytes, PayloadError&gt;&gt; + 'static, Global&gt;&gt;&gt; !Freeze for ClientResponse&lt;S&gt;","synthetic":true,"types":[]},{"text":"impl&lt;S, U&gt; Freeze for JsonBody&lt;S, U&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Freeze for MessageBody&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for Client","synthetic":true,"types":[]},{"text":"impl Freeze for SendClientRequest","synthetic":true,"types":[]},{"text":"impl !Freeze for WsClientError","synthetic":true,"types":[]},{"text":"impl Freeze for JsonPayloadError","synthetic":true,"types":[]},{"text":"impl !Freeze for TestResponse","synthetic":true,"types":[]},{"text":"impl !Freeze for WebsocketsRequest","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()