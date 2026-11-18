use std::fs;

use pump_shared::Graph;

fn main() -> std::io::Result<()> {
    let graph = Graph::standard();
    fs::create_dir_all("docs")?;
    fs::write("docs/therapy.mmd", graph.to_mermaid())?;
    fs::write("docs/therapy.dot", graph.to_dot())?;
    Ok(())
}
