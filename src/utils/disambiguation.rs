use anyhow::{bail, Result};
use dialoguer::Select;

#[derive(Debug, Clone)]
pub struct DisambiguationOption {
    pub id: String,
    pub name: String,
    pub scope: String,
}

pub fn choose(label: &str, options: &[DisambiguationOption]) -> Result<String> {
    match options.len() {
        0 => bail!("{label} not found"),
        1 => Ok(options[0].id.clone()),
        _ => {
            let rows = options
                .iter()
                .map(|option| format!("{}  •  {}  •  {}", option.name, option.scope, option.id))
                .collect::<Vec<_>>();
            let selected = Select::new()
                .with_prompt(format!("{label} is ambiguous. Pick one"))
                .items(&rows)
                .default(0)
                .interact()?;
            Ok(options[selected].id.clone())
        }
    }
}
