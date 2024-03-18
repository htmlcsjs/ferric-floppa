use std::collections::{hash_map::Entry, HashMap, HashSet};

use serenity::{
    all::{CreateEmbed, Message},
    async_trait,
};
use stoik::{formula::Molecule, StoikError};
use tracing::error;

use crate::{
    command::{CmdCtx, Command, FlopMessagable},
    Cli, FlopResult,
};

const HELP_MSG: &str = "Usage: stoik [FLAGS] ... [EQUATION] ...
Computes whether EQUATION is chemically balanced or not
  -h, --help Shows this";

#[derive(Debug)]
pub struct StoikCommand;

#[async_trait]
impl Command for StoikCommand {
    fn construct(_cli: &Cli, _data: &[u8]) -> FlopResult<Self> {
        Ok(Self)
    }

    async fn execute<'a>(&mut self, msg: &Message, ctx: CmdCtx<'a>) -> FlopResult<FlopMessagable> {
        let args = msg.content.trim_start_matches(ctx.command).trim();
        let mut equation = String::new();
        for arg in args.split(char::is_whitespace) {
            if arg == "--help" || arg == "-h" {
                return Ok(FlopMessagable::Text(HELP_MSG.to_string()));
            }
            equation = format!("{equation} {arg}");
        }
        equation = equation.trim().to_string();

        if !(equation.contains("->") || equation.contains("=>")) {
            return Ok(FlopMessagable::Text(
                "Products are not given, please use `=>` or `->` to seperate the two sides"
                    .to_string(),
            ));
        }

        let sanatised_eq = equation.replace("=>", "->");
        let (mut reactants_str, mut products_str) =
            sanatised_eq.split_once("->").unwrap_or_else(|| {
                error!("Could not split equation: `{equation}` after checking!");
                ("", "")
            });

        reactants_str = reactants_str.trim();
        products_str = products_str.trim();

        let mut reactants = Vec::new();
        for formula in reactants_str.split('+').map(|x| x.trim()) {
            match Molecule::from_formula(formula) {
                Ok(mol) => reactants.push((mol, formula.to_string())),
                Err(e) => {
                    return Ok(FlopMessagable::Text(format!(
                        "```{}```",
                        generate_stoik_error_msg(e, formula)
                    )));
                }
            }
        }

        let mut products = Vec::new();
        for formula in products_str.split('+').map(|x| x.trim()) {
            match Molecule::from_formula(formula) {
                Ok(mol) => products.push((mol, formula.to_string())),
                Err(e) => {
                    return Ok(FlopMessagable::Text(format!(
                        "```{}```",
                        generate_stoik_error_msg(e, formula)
                    )));
                }
            }
        }

        let mut lhs = HashMap::new();
        for mol in reactants {
            extend_mol_map(&mut lhs, mol.0.get_map());
        }

        let mut rhs = HashMap::new();
        for mol in products {
            extend_mol_map(&mut rhs, mol.0.get_map());
        }

        let mut keys = lhs.keys().collect::<HashSet<_>>();
        let mut balanced = HashMap::new();
        keys.extend(rhs.keys());

        for key in keys {
            balanced.insert(key.to_string(), lhs.get(key) == rhs.get(key));
        }

        let is_balanced = balanced.values().all(|x| *x);

        let lhs_results = balanced
            .keys()
            .map(|x| lhs.get(x).unwrap_or(&0))
            .fold(String::new(), |acc, x| format!("{acc}\n{x}"));
        let rhs_results = balanced
            .keys()
            .map(|x| rhs.get(x).unwrap_or(&0))
            .fold(String::new(), |acc, x| format!("{acc}\n{x}"));

        let embed = CreateEmbed::new()
            .field("Reactants", format!("```{lhs_results}```"), true)
            .field("Products", format!("```{rhs_results}```"), true)
            .field(
                "Balanced",
                format!(
                    "```{}```",
                    balanced
                        .iter()
                        .map(|x| if *x.1 { "✅✅\n" } else { "❌❌\n" })
                        .collect::<String>()
                ),
                true,
            )
            .title(if is_balanced {
                "✅ Your reaction is balanced ✅"
            } else {
                "❌ Your reaction is unbalanced ❌"
            });
        Ok(embed.into())
    }

    fn save(&self) -> Option<Vec<u8>> {
        None
    }
}

// maybe move to a pub func in stoik itself?
fn generate_stoik_error_msg(e: StoikError, formula: &str) -> String {
    match e {
        StoikError::InvalidToken(loc) => {
            loc.format_msg(formula, "Malformed formula", "Illegal token")
        }
        StoikError::NumberFirst(loc) => loc.format_msg(
            formula,
            "Malformed formula",
            "Compound groups cannot start with numbers",
        ),
        StoikError::UnpairedParenthesis(loc) => {
            loc.format_msg(formula, "Malformed formula", "Unpaired parenthesis")
        }
        StoikError::UnpairedBracket(loc) => {
            loc.format_msg(formula, "Malformed formula", "Unpaired bracket")
        }
        e => e.to_string(),
    }
}

fn extend_mol_map(main: &mut HashMap<String, i64>, mol: HashMap<String, i64>) {
    for (key, mol_val) in mol {
        if let Entry::Occupied(mut entry) = main.entry(key.clone()) {
            *entry.get_mut() += mol_val;
        } else {
            main.insert(key, mol_val);
        }
    }
}
