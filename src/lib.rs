extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree};
use quote::*;

struct Opts {
    sup: Option<String>,
    idtype: Option<TokenStream>,
}

macro_rules! fail {
    ($t:expr) => {
        {
            let failresult = quote!{
                compile_error!($t);
            };
            return failresult.into();
        }
    } 
}

macro_rules! consume_punct {
    ($i:expr) => {
        {
            let x  = $i.next();
            match x {
                Some(e) => {
                    match e {
                        TokenTree::Punct(_) => {},
                        TokenTree::Group(y) => {  
                            eprintln!("punctuation expected at {:?}", y.span());
                            fail!("while parsing actor");
                        },
                        TokenTree::Ident(y) => { 
                            eprintln!("punctuation expected at {:?}", y.span());
                            fail!("while parsing actor");
                        },
                        TokenTree::Literal(y) => {
                            eprintln!("punctuation expected at {:?}", y.span());
                            fail!("while parsing actor");
                        },
                    }
                },
                _ => fail!("something expected, found none")
            }
        }
    }
}

macro_rules! parse_type {
    ($i:expr) => {
        {
            let mut tokens : Vec<TokenTree> = Vec::new();
            let mut depth = 0;
            while let Some(e) = $i.next() {
                match e {
                    TokenTree::Ident(x) => {
                        tokens.push(TokenTree::Ident(x))
                    },
                    TokenTree::Punct(x) => {
                        match x.as_char() {
                            ',' => if depth == 0 {
                                break; 
                            },
                            '<' | '(' | '[' | '{' => { depth = depth +1 },
                            '>' | ')' | ']' | '}' => { depth = depth -1 }, 
                            _ => {} // neutral punct : 
                        } 
                        tokens.push(TokenTree::Punct(x))
                    },
                    TokenTree::Group(x) => tokens.push(TokenTree::Group(x)),
                    _ => {
                        eprintln!("your error is near {:?}", e);
                        fail!("unexpected token in actor state member type");
                    }
                }
            }
            TokenStream::from_iter(tokens.into_iter())
        }
    }
}

macro_rules! get_opt {
    ($si:expr) => {
        {
        let r = if let Some(t) = $si.next() {
            Some(t.to_string())
        } else {
            fail!("expected type");
        };
        consume_punct!($si);
        r
        }
    }    
}

macro_rules! parse_options {
    ($opts:expr,$i:expr) => { 
        {
            consume_punct!($i);
            if let Some(TokenTree::Group(block)) = $i.next() {
                let mut si = block.stream().into_iter();
                while let Some(e) = si.next() {
                    match e {
                        TokenTree::Ident(i) => {
                            consume_punct!(si);
                            match i.to_string().as_str() {
                                "sup" => {
                                    $opts.sup = get_opt!(si);
                                },
                                "id" => {
                                    $opts.idtype = Some(parse_type!(si));
                                },
                                _ => fail!("unexpected option")
                            }
                             
                        },
                        _ => fail!("bad syntax")
                    }
                }
            } else {
                fail!("expected block");
            }
            consume_punct!($i);
        }
    }
}
// Group { delimiter: Brace, stream: TokenStream [Ident { ident: "sup", span: #0 bytes(697..700) }, Punct { ch: ':', spacing: Alone, span: #0 bytes(700..701) }, Ident { ident: "None", span: #0 bytes(702..706) }, Punct { ch: ',', spacing: Alone, span: #0 bytes(706..707) }, Ident { ident: "id", span: #0 bytes(716..718) }, Punct { ch: ':', spacing: Alone, span: #0 bytes(718..719) }, Ident { ident: "u32", span: #0 bytes(720..723) }, Punct { ch: ',', spacing: Alone, span: #0 bytes(723..724) }], span: #0 bytes(687..730) }
//Some(Group { delimiter: Brace, stream: TokenStream [Ident { ident: "A", span: #0 bytes(756..757) }, Punct { ch: ',', spacing: Alone, span: #0 bytes(757..758) }, Ident { ident: "B", span: #0 bytes(76 7..768) }, Group { delimiter: Brace, stream: TokenStream [Ident { ident: "x", span: #0 bytes(783..784) }, Punct { ch: ':', spacing: Alone, span: #0 bytes(784..785) }, Punct { ch: '&', spacing: Alone , span: #0 bytes(786..787) }, Ident { ident: "str", span: #0 bytes(787..790) }, Punct { ch: ',', spacing: Alone, span: #0 bytes(790..791) }], span: #0 bytes(769..801) }, Punct { ch: ',', spacing: Al one, span: #0 bytes(801..802) }], span: #0 bytes(746..808) }) unexpected Punct { ch: ',', spacing: Alone, span: #0 bytes(808..809) }

macro_rules! get_block {
    ($i:expr) => {
        {
            consume_punct!($i);
            match $i.next() {
                Some(TokenTree::Group(block)) => {
                    block.stream().into_iter()
                },
                _ => fail!("expected block")
            }
        }
    }
}


macro_rules! parse_state {
    ($i:expr, $v: expr) => {
        {
            let mut ii = get_block!($i);
            while let Some(e) = ii.next() {
                match e {
                    TokenTree::Ident(name) => {
                        consume_punct!(ii); // :
                        $v.push((name.to_string(), parse_type!(ii)));
                    },
                    TokenTree::Punct(_) => {}, // ignore , 
                    _ => fail!("unexpected token in actor state definition")
                }
            }
        }
    }
}

macro_rules! parse_messages {
    ($i:expr, $v:expr) => {
        {
            let mut ii = get_block!($i);
            while let Some(e) = ii.next() {
                match e {
                    TokenTree::Ident(name) => {
                        match ii.next() {
                            Some(TokenTree::Punct(_)) => {
                                // bare variant
                                $v.push((name.to_string(), None))
                            },
                            Some(TokenTree::Group(group)) => {
                                $v.push((name.to_string(), Some(group.stream())))
                            },
                            _ => fail!("unexpected token after message identifier")
                        }
                    },
                    TokenTree::Punct(_) => {}, // ignore trailing ,
                    _ => fail!("unexpected token in messages definition")
                }
            }
        }
    }
}


#[proc_macro]
pub fn actor(tokens: TokenStream) -> TokenStream {
    let mut input = tokens.into_iter();
    let basename = match input.next() {
        Some(TokenTree::Ident(i)) => {
            i.to_string()
        },
        x => {
            eprintln!("w0t ? {:?}", x);
            fail!("actor needs a name");
        }
    };
    
    let actor_ident = format_ident!("{}Actor", basename);
    let messages_ident = format_ident!("{}ActorMessages", basename);
    let state_ident = format_ident!("{}ActorState", basename);

    let preamble = quote!{
        #[derive(Debug)]
        struct #actor_ident {}
    };

    let mut opts = Opts{
        sup: None,
        idtype: None,
    };
  
    let mut messages_structs = quote!{};
    let mut messages_block = quote!{};
    let mut extra_state : Vec<(String,TokenStream)> = Vec::new();
    let mut messages : Vec<(String, Option<TokenStream>)> = Vec::new();
    let mut messages_match_block = quote!{};

    while let Some(e) = input.next() {
        match e {
            TokenTree::Ident(i) => {
                match i.to_string().as_str() {
                    "Options" => parse_options!(opts, input),
                    "Messages" => parse_messages!(input, messages),
                    "State" => parse_state!(input, extra_state),
                    _ => fail!("unexpected key in actor")
                }
            },
            TokenTree::Punct(_) => {}, // ignore commas at this level
            _ => {
                eprintln!("unexpected {:?}", e);
                fail!("while parsing actor");
            }
        }
    }
    
    let mut start_params = quote! {};
    let mut state_opts_block = quote! {};
    let mut state_init_opts_block = quote! {};

    if let Some(t) = opts.sup {
        let mt = format_ident!("{}ActorMessages", t);
        start_params = quote! { #start_params , sup_ch: Option<mpsc::Sender<#mt>> };
        state_opts_block = quote! { #state_opts_block  sup_ch: Option<mpsc::Sender<#mt>>, };
        state_init_opts_block = quote! { #state_init_opts_block  sup_ch: sup_ch, };
    };

    if let Some(t) = opts.idtype {
        #[allow(unused_assignments)]
        let mut bt = quote! {};
        bt = t.into();
        start_params = quote! { #start_params , id: &#bt };
        state_opts_block = quote! { #state_opts_block id: #bt, };
        state_init_opts_block = quote! { #state_init_opts_block  id: id.clone(), };
    };

    for (name, group) in messages {
        let n = format_ident!("{}", name);
        let sn = format_ident!("{}{}", messages_ident, name);
        let hn = format_ident!("handle_{}", name.to_lowercase());
        if let Some(g) = group {
        #[allow(unused_assignments)]
            let mut x = quote!{};
            x = g.into(); // coerse into token_macro2
            messages_structs = quote! { #messages_structs 
                #[derive(Debug)]
                struct #sn { #x } 
            };
            messages_block = quote! { #messages_block #n(#sn), };
            messages_match_block = quote! { #messages_match_block 
                #messages_ident::#n(payload) => self.#hn(state, payload).await,
            }
        } else {
            messages_block = quote! { #messages_block #n, };
            messages_match_block = quote! { #messages_match_block 
                #messages_ident::#n => self.#hn(state).await,
            }
            
        }
    }

    for (name, typ) in extra_state {
        let n = format_ident!("{}", name);
        #[allow(unused_assignments)]
        let mut t = quote! {};
        t = typ.into();
        state_opts_block = quote! { #state_opts_block #n : Option<#t> , };
        state_init_opts_block = quote! { #state_init_opts_block #n : None , };
    }
 
    quote!{
        #preamble

        #messages_structs
        
        #[derive(Debug)]
        enum #messages_ident {
            #messages_block
        }

        #[derive(Debug)]
        struct #state_ident {
            rx: mpsc::Receiver<#messages_ident>,
            tx: mpsc::Sender<#messages_ident>,
            #state_opts_block
        }

        impl #actor_ident {
            fn start(mut self #start_params) -> mpsc::Sender<#messages_ident> {
                let (tx, rx) = mpsc::channel(100); 
                let mut state = #state_ident {
                    rx: rx,
                    tx: tx.clone(),
                    #state_init_opts_block
                };
                tokio::spawn(async move {
                    while let Some(msg) = state.rx.recv().await {
                        self.handle(&mut state, msg);
                    }
                });
                tx 
            }

          fn handle(&self, state: &mut #state_ident, msg: #messages_ident) -> () {
            match msg {
                #messages_match_block
            }
            ()
          }
        }
    }.into()
}


