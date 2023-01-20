use crate::plugins::InstallConfig;
use crate::plugins::Plugin;
use crate::utils::fs;
use crate::utils::logger::add_file_msg;
use anyhow::Result;
use rust_embed::RustEmbed;

pub struct Container {}

#[derive(RustEmbed)]
#[folder = "template-plugin-container"]
struct Asset;

impl Plugin for Container {
    fn name(&self) -> &'static str {
        "Container"
    }

    fn install(&self, install_config: InstallConfig) -> Result<()> {
        for filename in Asset::iter() {
            let file_contents = Asset::get(filename.as_ref()).unwrap();
            let mut file_path = std::path::PathBuf::from(&install_config.project_dir);
            file_path.push(filename.as_ref());
            let mut directory_path = std::path::PathBuf::from(&file_path);
            directory_path.pop();

            add_file_msg(filename.as_ref());
            std::fs::create_dir_all(directory_path)?;
            std::fs::write(file_path, file_contents.data)?;
        }

        // TODO: Fix these appends/prepends by prepending the filepath with project_dir
        // currently, this works because we assume the current working directory is the project's root
        fs::append(
            "README.md",
            r##"
# Containerize your application
      
## Building a container
`docker build -t image-name .`

## Running the container
`docker run -e SECRET_KEY=123 -e DATABASE_URL=postgres://postgres:postgres@localhost/database -p 3000:3000 image-name`

"##,
        )?;

        Ok(())
    }
}
