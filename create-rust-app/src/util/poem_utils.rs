use std::sync::Mutex;

use poem::http::{StatusCode, Uri};
use poem::middleware::{AddData, AddDataEndpoint};
use poem::web::Data;
use poem::{handler, Body, EndpointExt, IntoResponse, Response, Route};
use tera::Context;

use crate::util::template_utils::{to_template_name, DEFAULT_TEMPLATE, TEMPLATES};

use super::template_utils::SinglePageApplication;

///  view: the template which renders the app
///
///  Full example (to render the `views/spa.html` template):
/// ```
///  app = app.nest("/my-spa", create_rust_app::render_single_page_application("spa.html"));
/// ```
pub fn render_single_page_application(view: &str) -> AddDataEndpoint<Route, SinglePageApplication> {
    let view = view.strip_prefix("/").unwrap_or(view);

    Route::new()
        .at("*", poem::get(render_spa_handler))
        .with(AddData::new(SinglePageApplication {
            view_name: view.to_string(),
        }))
}

#[handler]
async fn render_spa_handler(spa_info: Data<&SinglePageApplication>) -> impl IntoResponse {
    let content = TEMPLATES
        .render(spa_info.view_name.as_str(), &Context::new())
        .unwrap();
    template_response(content)
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
#[handler]
pub async fn render_views(uri: &Uri) -> impl IntoResponse {
    let path = uri.path();

    #[cfg(debug_assertions)]
    {
        // Catch viteJS ping requests and try to handle them gracefully
        // Request the browser to refresh the page (maybe the server is up but the browser just can't reconnect)

        if path.eq("/__vite_ping") {
            #[cfg(feature = "plugin_dev")]
            {
                crate::dev::vitejs_ping_down().await;
            }
            let mut count = REQUEST_REFRESH_COUNT.lock().unwrap();
            if *count < 3 {
                *count = 1 + *count;
                println!("The vite dev server seems to be down... refreshing page ({count}).");
                return poem::web::Redirect::temporary(".").into_response();
            } else {
                println!("The vite dev server is down.");
                return StatusCode::NOT_FOUND.into_response();
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

    let mut template_path = to_template_name(path);
    let mut content_result = TEMPLATES.render(template_path, &Context::new());

    if content_result.is_err() {
        #[cfg(debug_assertions)]
        {
            // dev asset serving
            let asset_path = &format!("./frontend{path}");
            if std::path::PathBuf::from(asset_path).is_file() {
                println!("ASSET_FILE {path} => {asset_path}");

                return file_response(asset_path).await;
            }

            let public_path = &format!("./frontend/public{path}");
            if std::path::PathBuf::from(public_path).is_file() {
                println!("PUBLIC_FILE {path} => {public_path}");

                return file_response(public_path).await;
            }
        }

        #[cfg(not(debug_assertions))]
        {
            // production asset serving
            let static_path = &format!("./frontend/dist{path}");
            if std::path::PathBuf::from(static_path).is_file() {
                return file_response(static_path).await;
            }
        }

        content_result = TEMPLATES.render(DEFAULT_TEMPLATE, &Context::new());
        template_path = DEFAULT_TEMPLATE;
        if content_result.is_err() {
            // default template doesn't exist -- return 404 not found
            return StatusCode::NOT_FOUND.into();
        }
    }

    println!("TEMPLATE_FILE {path} => {template_path}");

    let content = content_result.unwrap();

    template_response(uri, content)
}

fn template_response(uri: &Uri, content: String) -> Response {
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

    Response::builder()
        .status(StatusCode::OK)
        .content_type("text/html")
        .body(content)
}

async fn file_response(path: &String) -> Response {
    let file = tokio::fs::read(path).await.unwrap();
    let content_type = mime_guess::from_path(path).first_raw();
    let mut response = Response::builder();
    if content_type.is_some() {
        response = response.content_type(content_type.unwrap());
    }
    response = response.status(StatusCode::OK);
    response.body(Body::from(file))
}
