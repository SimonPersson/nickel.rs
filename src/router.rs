use http::server::{Request, ResponseWriter};
use regex::Regex;

/// A Route is the basic data structure that stores both the path
/// and the handler that gets executed for the route.
/// The path can contain variable pattern such as `user/:userid/invoices`
struct Route {
    pub path: String,
    pub handler: fn(request: &Request, response: &mut ResponseWriter),
    matcher: Regex
}

impl Clone for Route {
    fn clone(&self) -> Route {
        Route { path: self.path.clone(), handler: self.handler, matcher: self.matcher.clone() }
    }
}

/// The RouteRegexFactory is responsible to convert paths to Regex patterns to
/// match against concrete URLs
struct RouteRegexFactory;

impl RouteRegexFactory {
    fn create_regex (route_path: &str) -> Regex {

        static VALID_SEQUENCE:&'static str  = ".[a-zA-Z0-9_-]*";
        static REGEX_START:&'static str     = "^";
        static REGEX_END:&'static str       = "$";

        // this should better be a regex! macro but I couldn't get it to work
        let regex = match Regex::new(r":[a-zA-Z0-9_-]*") {
            Ok(re) => re,
            Err(err) => fail!("{}", err)
        };

        let result = REGEX_START.to_string()
                                .append(regex.replace_all(route_path, VALID_SEQUENCE).as_slice())
                                .append(REGEX_END);

        match Regex::new(result.as_slice()) {
            Ok(re) => re,
            Err(err) => fail!("{}", err)
        }
    }
}

/// The Router's job is it to hold routes and to resolve them later against
/// concrete URLs

#[deriving(Clone)]
pub struct Router{
    pub routes: Vec<Route>,
}

impl Router {
    pub fn new () -> Router {
        Router {
            routes: Vec::new()
        }
    }

    pub fn add_route (&mut self, path: String, handler: fn(request: &Request, response: &mut ResponseWriter)) -> () {
        let matcher = RouteRegexFactory::create_regex(path.as_slice());
        let route = Route {
            path: path,
            matcher: matcher,
            handler: handler
        };
        self.routes.push(route);
    }

    pub fn match_route<'a>(&'a self, path: String) -> Option<&'a Route> {
        self.routes.iter().find(|item| item.matcher.is_match(path.as_slice()))
    }
}


#[test]
fn creates_valid_regex_for_var_routes () {
    let regex = RouteRegexFactory::create_regex("foo/:uid/bar/:groupid");
    assert_eq!(regex.is_match("foo/4711/bar/5490"), true);
    assert_eq!(regex.is_match("foo/"), false);
}

#[test]
fn can_match_var_routes () {
    let route_store = &mut Router::new();

    fn handler (request: &Request, response: &mut ResponseWriter) -> () {
        response.write("hello from foo".as_bytes()); 
    };

    route_store.add_route("/foo/:userid".to_string(), handler);
    route_store.add_route("/bar".to_string(), handler);
    
    let route = route_store.match_route("/foo/4711".to_string());

    let result = match route {
        Some(re) => true,
        None => false
    };

    assert_eq!(result, true);

    let route = route_store.match_route("/bar/4711".to_string());

    let result = match route {
        Some(re) => true,
        None => false
    };

    assert_eq!(result, false);

    let route = route_store.match_route("/foo".to_string());

    let result = match route {
        Some(re) => true,
        None => false
    };

    assert_eq!(result, false);
}