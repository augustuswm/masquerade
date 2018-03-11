use staticfile::Static;

use std::path::Path;

pub fn v1() -> Static {
    Static::new(Path::new("src/frontend/static/"))
}
