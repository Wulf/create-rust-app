use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, Scope, web};
use actix_web::http::StatusCode;
use tera::Context;
use super::template_utils::SinglePageApplication;
use crate::util::template_utils::{DEFAULT_TEMPLATE, TEMPLATES, to_template_name};

trait Hostname {
    fn hostname(&self) -> String;
}

impl Hostname for HttpRequest {
    fn hostname(&self) -> String {
        self.connection_info().host().split(":").next().expect("Could not extract host information for request").to_string()
    }
}

/// 'route': the route where the SPA should be served from, for example: "/app"
/// 'view': the view which renders the SPA, for example: "spa/index.html"
pub fn render_single_page_application(route: &str, view: &str) -> Scope {
    use actix_web::web::Data;

    let route = route.strip_prefix("/").unwrap_or(route);
    let view = view.strip_prefix("/").unwrap_or(view);

    actix_web::web::scope(&format!("/{}{{tail:(/.*)?}}", route))
        .app_data(Data::new(SinglePageApplication {
            view_name: view.to_string()
        }))
        .route("", web::get().to(render_spa_handler))
}

async fn render_spa_handler(req: HttpRequest, spa_info: web::Data<SinglePageApplication>) -> HttpResponse {
    let content = TEMPLATES.render(spa_info.view_name.as_str(), &Context::new()).unwrap();

    template_response(content, #[cfg(debug_assertions)] req.hostname())
}

pub async fn render_views(req: HttpRequest) -> HttpResponse {
    let path = req.path();
    let ctx = Context::new();

    #[cfg(debug_assertions)]
    if path.eq("/__vite_ping") {
        println!("The vite dev server seems to be down...");
        return HttpResponse::NotFound().finish();
    }

    let mut template_path = to_template_name(req.path());
    let mut content_result = TEMPLATES.render(template_path, &ctx);

    if content_result.is_err() {
        #[cfg(debug_assertions)] {
            // dev asset serving
            let asset_path = &format!("./frontend{path}");
            if std::path::PathBuf::from(asset_path).is_file() {
                println!("ASSET_FILE {path} => {asset_path}");
                return NamedFile::open(asset_path).unwrap().into_response(&req)
            }

            let public_path = &format!("./frontend/public{path}");
            if std::path::PathBuf::from(public_path).is_file() {
                println!("PUBLIC_FILE {path} => {public_path}");
                return NamedFile::open(public_path).unwrap().into_response(&req)
            }
        }

        #[cfg(not(debug_assertions))] {
            // production asset serving
            let static_path = &format!("./frontend/dist{path}");
            if std::path::PathBuf::from(static_path).is_file() {
                return NamedFile::open(static_path).unwrap().into_response(&req);
            }
        }

        content_result = TEMPLATES.render(DEFAULT_TEMPLATE, &ctx);
        template_path = DEFAULT_TEMPLATE;
        if content_result.is_err() {
            // default template doesn't exist -- return 404 not found
            return HttpResponse::NotFound().finish()
        }
    }

    println!("TEMPLATE_FILE {path} => {template_path}");

    let content = content_result.unwrap();

    template_response(content, #[cfg(debug_assertions)] req.hostname())
}

fn template_response(content: String, #[cfg(debug_assertions)] host: String) -> HttpResponse {
    let mut content = content;
    #[cfg(debug_assertions)] {
        let inject = format!(r##"
        <!-- development mode -->
        <script type="module">
            import RefreshRuntime from 'http://{host}:21012/@react-refresh'
            RefreshRuntime.injectIntoGlobalHook(window)
            window.$RefreshReg$ = () => {{}}
            window.$RefreshSig$ = () => (type) => type
            window.__vite_plugin_react_preamble_installed__ = true
        </script>
        <script type="module" src="http://{host}:21012/src/dev.tsx"></script>
        "##);

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
