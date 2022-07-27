use proc_macro::Delimiter::Parenthesis;
use proc_macro::Delimiter::Bracket;
use proc_macro::Delimiter::Brace;
use proc_macro::TokenStream;
use proc_macro::TokenTree as Tree;
use proc_macro::Literal;
use proc_macro::Spacing;
use proc_macro::Punct;
use proc_macro::Ident;
use proc_macro::Group;
use proc_macro::Span;

use std::env;

#[proc_macro_attribute]
pub fn smartlink(attr: TokenStream, item: TokenStream) -> TokenStream {
    let env_var = match attr.is_empty() {
        true => "SMARTLINK_NO_IMPL".to_string(),
        false => attr.to_string(),
    };
    if let Ok(shared_object) = env::var(env_var) {
        let (fn_name, arg_names, fn_signature, p_index) = parse_signature(item);

        let mut body = Vec::new();

        add_link_attr(&mut body, &shared_object);
        body.push(Tree::Ident(Ident::new("extern", Span::call_site())));
        body.push(Tree::Literal(Literal::string("Rust")));

        // the import
        let imports_stream = TokenStream::from_iter({
            let mut import_body = vec![
                Tree::Ident(Ident::new("fn", Span::call_site())),
                Tree::Ident(Ident::new(&fn_name, Span::call_site())),
            ];
            import_body.extend_from_slice(&fn_signature[p_index..]);
            import_body.push(Tree::Punct(Punct::new(';', Spacing::Alone)));
            import_body
        });
        body.push(Tree::Group(Group::new(Brace, imports_stream)));

        // the call
        let params_stream = TokenStream::from_iter({
            let mut call_params = Vec::new();
            for arg in arg_names {
                call_params.push(Tree::Ident(Ident::new(&arg, Span::call_site())));
                call_params.push(Tree::Punct(Punct::new(',', Spacing::Alone)));
            }
            call_params
        });
        let unsafe_block = TokenStream::from_iter(vec![
            Tree::Ident(Ident::new(&fn_name, Span::call_site())),
            Tree::Group(Group::new(Parenthesis, params_stream)),
        ]);
        body.push(Tree::Ident(Ident::new("unsafe", Span::call_site())));
        body.push(Tree::Group(Group::new(Brace, unsafe_block)));

        let mut definition = fn_signature.clone();
        let body_stream = TokenStream::from_iter(body);
        let group = Group::new(Brace, body_stream);
        definition.push(Tree::Group(group));

        let retval = TokenStream::from_iter(definition);

        #[cfg(feature = "show_output")]
        println!("{}", retval.to_string());

        retval
    } else {
        item
    }
}

fn parse_arguments(item: TokenStream) -> Vec<String> {
    let mut arguments = Vec::new();
    let mut candidate = None;

    for tree in item {
        if let Tree::Ident(ident) = tree {
            let string = ident.to_string();
            if &string == "self" {
                arguments.push(string);
            } else if &string == "_" {
                panic!("`_` parameter names are unsupported by smartlink");
            } else {
                candidate = Some(string);
            }
        } else if let Tree::Punct(punct) = tree {
            if punct.as_char() == ':' {
                if punct.spacing() == Spacing::Alone {
                    // specified Alone to avoid `crate::MyType` wrong positives
                    if let Some(candidate) = candidate.take() {
                        arguments.push(candidate);
                    }
                } else {
                    candidate = None;
                }
            }
        }
    }

    arguments
}

fn parse_signature(item: TokenStream) -> (String, Vec<String>, Vec<Tree>, usize) {
    let mut arg_names = None;
    let mut fn_name = None;
    let mut fn_signature = Vec::new();
    let mut can_read_fn_name = false;
    let mut p_index = 0;

    for tree in item {
        if fn_name.is_none() {
            p_index += 1;
        }

        if let Tree::Ident(ident) = &tree {

            let mut ident = ident.to_string();
            if can_read_fn_name && fn_name.is_none() {
                ident = "_ZN7testplz16example_function17hb1b493dba10a26ddE".to_string();
                fn_name = Some(ident);
            } else if ident == "fn" {
                can_read_fn_name = true;
            }

        } else if let Tree::Group(group) = &tree {

            if group.delimiter() == Brace {
                // we've reached the function's body
                break;
            } else if group.delimiter() == Parenthesis {
                if arg_names.is_none() && fn_name.is_some() {
                    arg_names = Some(parse_arguments(group.stream()));
                }
            }

        }

        fn_signature.push(tree);
    }

    (fn_name.unwrap(), arg_names.unwrap(), fn_signature, p_index)
}

fn add_link_attr(body: &mut Vec<Tree>, shared_object: &str) {
    let in_par = vec![
        Tree::Ident(Ident::new("name", Span::call_site())),
        Tree::Punct(Punct::new('=', Spacing::Alone)),
        Tree::Literal(Literal::string(shared_object)),
        Tree::Punct(Punct::new(',', Spacing::Alone)),
        Tree::Ident(Ident::new("kind", Span::call_site())),
        Tree::Punct(Punct::new('=', Spacing::Alone)),
        Tree::Literal(Literal::string("dylib")),
    ];

    let in_par_stream = TokenStream::from_iter(in_par);
    let in_bracket = vec![
        Tree::Ident(Ident::new("link", Span::call_site())),
        Tree::Group(Group::new(Parenthesis, in_par_stream)),
    ];

    let in_bracket_stream = TokenStream::from_iter(in_bracket);
    body.push(Tree::Punct(Punct::new('#', Spacing::Alone)));
    body.push(Tree::Group(Group::new(Bracket, in_bracket_stream)));
}
