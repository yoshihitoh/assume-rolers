use std::io;

use skim::prelude::{SkimItemReader, SkimOptionsBuilder};
use skim::Skim;

use crate::profile::select::SelectProfile;
use crate::profile::{Profile, ProfileSet};

pub struct SkimProfileSelector;

impl SelectProfile for SkimProfileSelector {
    fn select_profile<'a>(&self, profiles: &'a ProfileSet) -> anyhow::Result<Option<&'a Profile>> {
        let mut names = profiles
            .profiles()
            .filter(|&p| p.role_arn.is_some())
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>();
        names.sort();

        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(io::Cursor::new(names.join("\n")));

        let options = SkimOptionsBuilder::default().reverse(true).build()?;
        let selected = Skim::run_with(&options, Some(items))
            .and_then(|out| (!out.is_abort).then_some(out.selected_items))
            .unwrap_or_default();

        let selected_name = selected.into_iter().next().map(|x| x.output().to_string());
        Ok(selected_name.and_then(|name| profiles.get_profile(&name)))
    }
}
