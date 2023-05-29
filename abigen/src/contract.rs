// Copyright 2015-2019 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use proc_macro2::TokenStream;
use quote::quote;

// use crate::{constructor::Constructor,};
use crate::{build::AbiExtension, event::Event, function::Function};

/// Structure used to generate rust interface for solidity contract.
pub struct Contract {
    contract_name: Option<String>,
    // constructor: Option<Constructor>,
    functions: Vec<Function>,
    events: Vec<Event>,
    extension: Option<AbiExtension>,
}

impl<'a> From<&'a ethabi::Contract> for Contract {
    fn from(c: &'a ethabi::Contract) -> Self {
        let mut events: Vec<_> = c
            .events
            .values()
            .flat_map(|events| {
                let count = events.len();

                events.iter().enumerate().map(move |(index, event)| {
                    if count <= 1 {
                        (&event.name, event).into()
                    } else {
                        (&format!("{}{}", event.name, index + 1), event).into()
                    }
                })
            })
            .collect();

        // Since some people will actually commit this code, we use a "stable" generation order
        events.sort_by(|left: &Event, right: &Event| left.name.cmp(&right.name));

        let mut functions: Vec<_> = c
            .functions
            .values()
            .flat_map(|functions| {
                let count = functions.len();

                functions.iter().enumerate().map(move |(index, function)| {
                    if count <= 1 {
                        (&function.name, function).into()
                    } else {
                        (&format!("{}{}", function.name, index + 1), function).into()
                    }
                })
            })
            .collect();

        // Since some people will actually commit this code, we use a "stable" generation order
        functions.sort_by(|left: &Function, right: &Function| left.name.cmp(&right.name));

        Contract {
            // constructor: c.constructor.as_ref().map(Into::into),
            functions,
            events,
            extension: None,
            contract_name: None,
        }
    }
}

impl Contract {
    pub fn add_contract_name(mut self, name: String) -> Self {
        self.contract_name = Some(name);
        self
    }

    pub fn add_extension(mut self, extension: Option<AbiExtension>) -> Self {
        if let Some(extension) = extension {
            let event_extension = extension.event_extension();
            self.extension = Some(extension);

            self.events.iter_mut().for_each(|event| {
                event.add_extension(event_extension.clone());
            });
        }
        self
    }
    /// Generates rust interface for a contract.
    pub fn generate(&self) -> TokenStream {
        // let constructor = self.constructor.as_ref().map(Constructor::generate);
        let functions: Vec<_> = self.functions.iter().map(Function::generate).collect();
        let events: Vec<_> = self
            .events
            .iter()
            .map(|event| event.generate_event())
            .collect();

        let events_ident: Vec<_> = self
            .events
            .iter()
            .map(|event| event.generate_camel_name())
            .collect();
        // let logs: Vec<_> = self.events.iter().map(Event::generate_log).collect();

        let event_match: Vec<_> = self
            .events
            .iter()
            .map(|event| {
                let event = event.generate_camel_name();
                quote! {
                    if let Some(event) = #event::match_and_decode(log) {
                        return Some(Events::#event(event));
                    }
                }
            })
            .collect();

        let derive = if let Some(extension) = &self.extension {
            let event_extension = extension.event_extension();
            let list = event_extension.extended_event_derive();
            if list.len() > 0 {
                let ident: Vec<_> = list
                    .iter()
                    .map(|ident| syn::parse_str::<syn::Path>(ident).unwrap())
                    .collect();
                Some(quote! {
                    #[derive(#(#ident),*)]
                })
            } else {
                None
            }
        } else {
            None
        };

        let contract_name = self.contract_name.clone().unwrap_or("".to_string()).to_string();


        quote! {

            const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";
            const CONTRACT_NAME: &'static str = #contract_name;

            // #constructor

            /// Contract's functions.
            #[allow(dead_code, unused_imports, unused_variables)]
            pub mod functions {
                use super::INTERNAL_ERR;
                #(#functions)*
            }

            /// Contract's events.
            #[allow(dead_code, unused_imports, unused_variables)]
            pub mod events {
                use super::INTERNAL_ERR;

                #derive
                pub enum Events {
                    #( #events_ident(#events_ident), )*
                }


                impl Events {
                    pub fn match_and_decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Option<Events> {
                        use substreams_ethereum::Event;
                        #( #event_match )*
                        return None
                    }
                }

                #(#events)*
            }
        }
    }
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::assertions::assert_ast_eq;

    use super::Contract;

    #[test]
    fn test_no_body() {
        let ethabi_contract = ethabi::Contract {
            constructor: None,
            functions: Default::default(),
            events: Default::default(),
            errors: Default::default(),
            receive: false,
            fallback: false,
        };

        let c = Contract::from(&ethabi_contract);

        assert_ast_eq(
            c.generate(),
            quote! {
                const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";

                /// Contract's functions.
                #[allow(dead_code, unused_imports, unused_variables)]
                pub mod functions {
                    use super::INTERNAL_ERR;
                }

                /// Contract's events.
                #[allow(dead_code, unused_imports, unused_variables)]
                pub mod events {
                    use super::INTERNAL_ERR;
                }
            },
        );
    }
}
