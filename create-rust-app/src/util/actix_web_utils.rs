use std::str::FromStr;
use std::sync::Mutex;

use super::template_utils::SinglePageApplication;
use crate::util::template_utils::{to_template_name, DEFAULT_TEMPLATE, TEMPLATES};
use actix_files::NamedFile;
use actix_http::Uri;
use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, Scope};
use tera::Context;

/// 'route': the route where the SPA should be served from, for example: "/app"
/// 'view': the view which renders the SPA, for example: "spa/index.html"
pub fn render_single_page_application(route: &str, view: &str) -> Scope {
    use actix_web::web::Data;

    let route = route.strip_prefix('/').unwrap_or(route);
    let view = view.strip_prefix('/').unwrap_or(view);

    actix_web::web::scope(&format!("/{route}{{tail:(/.*)?}}"))
        .app_data(Data::new(SinglePageApplication {
            view_name: view.to_string(),
        }))
        .route("", web::get().to(render_spa_handler))
}

async fn render_spa_handler(
    req: HttpRequest,
    spa_info: web::Data<SinglePageApplication>,
) -> HttpResponse {
    let content = TEMPLATES
        .render(spa_info.view_name.as_str(), &Context::new())
        .unwrap();
    template_response(req, content)
}

// used to count number of refresh requests sent when viteJS dev server is down
#[cfg(debug_assertions)]
static REQUEST_REFRESH_COUNT: Mutex<i32> = Mutex::new(0);

/// takes a request to, say, www.you_webapp.com/foo/bar and looks in the ./backend/views folder
/// for a html file/template at the matching path (in this case, ./foo/bar.html),
/// defaults to index.html
///
/// then, your frontend (all the css files, scripts, etc. in your frontend's vite manifest (at ./frontend/dist/manifest.json))
/// will be compiled and injected into the template wherever `{{ bundle(name="index.tsx") }}` is (the `index.tsx` can be any .tsx file in ./frontend/bundles)
///
/// then, that compiled html is sent to the client
///
/// NOTE the frontend/dist/manifest.json file referenced is generated in the frontend when it compiles
pub async fn render_views(req: HttpRequest) -> HttpResponse {
    let path = req.path();

    #[cfg(debug_assertions)]
    {
        if path.eq("/__vite_ping") {
            println!("The vite dev server seems to be down...");
        }

        // Catch viteJS ping requests and try to handle them gracefully
        // Request the browser to refresh the page (maybe the server is up but the browser just can't reconnect)

        if path.eq("/__vite_ping") {
            #[cfg(feature = "plugin_dev")]
            {
                crate::dev::vitejs_ping_down().await;
            }
            let mut count = REQUEST_REFRESH_COUNT.lock().unwrap();
            if *count < 3 {
                *count += 1;
                println!("The vite dev server seems to be down... refreshing page ({count}).");
                return HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
                    .append_header(("Location", "."))
                    .finish();
            } else {
                println!("The vite dev server is down.");
                return HttpResponse::NotFound().finish();
            }
        }
        // If this is a non-viteJS ping request, let's reset the refresh attempt count
        else {
            #[cfg(feature = "plugin_dev")]
            {
                crate::dev::vitejs_ping_up().await;
            }
            let mut count = REQUEST_REFRESH_COUNT.lock().unwrap();
            *count = 0;
        }
    }

    let mut template_path = to_template_name(req.path());
    // try and render from your ./backend/views
    let mut content_result = TEMPLATES.render(template_path, &Context::new());

    // if that fails, then
    //  if in debug mode look for views in ./frontend/... or ./frontend/public/...
    //  else default to ./backend/views/index.html
    if content_result.is_err() {
        #[cfg(debug_assertions)]
        {
            // dev asset serving
            let asset_path = &format!("./frontend{path}");
            if std::path::PathBuf::from(asset_path).is_file() {
                println!("ASSET_FILE {path} => {asset_path}");
                return NamedFile::open(asset_path).unwrap().into_response(&req);
            }

            let public_path = &format!("./frontend/public{path}");
            if std::path::PathBuf::from(public_path).is_file() {
                println!("PUBLIC_FILE {path} => {public_path}");
                return NamedFile::open(public_path).unwrap().into_response(&req);
            }
        }

        #[cfg(not(debug_assertions))]
        {
            // production asset serving
            let static_path = &format!("./frontend/dist{path}");
            if std::path::PathBuf::from(static_path).is_file() {
                return NamedFile::open(static_path).unwrap().into_response(&req);
            }
        }

        content_result = TEMPLATES.render(DEFAULT_TEMPLATE, &Context::new());
        template_path = DEFAULT_TEMPLATE;
        if content_result.is_err() {
            // default template doesn't exist -- return 404 not found
            return HttpResponse::NotFound().finish();
        }
    }

    println!("TEMPLATE_FILE {path} => {template_path}");

    let content = content_result.unwrap();

    template_response(req, content)
}

fn template_response(req: HttpRequest, content: String) -> HttpResponse {
    let mut content = content;
    #[cfg(debug_assertions)]
    {
        let uri = Uri::from_str(req.connection_info().host());
        let hostname = match &uri {
            Ok(uri) => uri.host().unwrap_or("localhost"),
            Err(_) => "localhost",
        };

        let inject: &str = &format!(
            r##"
        <!-- development mode -->
        <script type="module">
            import RefreshRuntime from 'http://{hostname}:21012/@react-refresh'
            RefreshRuntime.injectIntoGlobalHook(window)
            window.$RefreshReg$ = () => {{}}
            window.$RefreshSig$ = () => (type) => type
            window.__vite_plugin_react_preamble_installed__ = true
        </script>
        <script type="module" src="http://{hostname}:21012/src/dev.tsx"></script>
        "##
        );

        if content.contains("<body>") {
            content = content.replace("<body>", &format!("<body>{inject}"));
        } else {
            content = format!("{inject}{content}");
        }
    }

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html")
        .body(content)
}
