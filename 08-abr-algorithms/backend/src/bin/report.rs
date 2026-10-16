use abr::player::PlayerConfig;
use abr::report::markdown_table;
use abr::trace::bundled;

fn main() -> anyhow::Result<()> {
    print!("{}", markdown_table(&PlayerConfig::default(), &bundled()?)?);
    Ok(())
}
