use cao_lang::prelude::SubProgramType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Card<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub ty: SubProgramType,
    pub input: Vec<&'a str>,
    pub output: Vec<&'a str>,
    pub constants: Vec<&'a str>,
}

impl<'a> Card<'a> {
    pub fn from_str_parts(
        name: &'a str,
        description: &'a str,
        ty: SubProgramType,
        input: &'a [&'a str],
        output: &'a [&'a str],
        constants: &'a [&'a str],
    ) -> Self {
        Self {
            name,
            description,
            ty,
            input: input.iter().map(|x| *x).collect(),
            output: output.iter().map(|x| *x).collect(),
            constants: constants.iter().map(|x| *x).collect(),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Schema<'a> {
    pub cards: Vec<Card<'a>>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct OwnedCard {
    pub name: String,
    pub description: String,
    pub ty: SubProgramType,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub constants: Vec<String>,
}

impl OwnedCard {
    pub fn as_card(&self) -> Card {
        Card {
            name: self.name.as_str(),
            description: self.description.as_str(),
            ty: self.ty,
            input: self.input.iter().map(|s| s.as_str()).collect(),
            output: self.output.iter().map(|s| s.as_str()).collect(),
            constants: self.constants.iter().map(|s| s.as_str()).collect(),
        }
    }
}
