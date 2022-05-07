extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree, token_stream};
use quote::*;

struct Opts {
    sup: Option<String>,
    idtype: Option<String>,
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


macro_rules! get_opt {
    ($si:expr) => {
        if let Some(t) = $si.next() {
            Some(t.to_string())
        } else {
            fail!("expected type");
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
                                    $opts.idtype = get_opt!(si);
                                },
                                _ => fail!("unexpected option")
                            }
                            consume_punct!(si);
                             
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



macro_rules! parse_block {
    ($i:expr) => {
        {
            consume_punct!($i);
            let r = match $i.next() {
                Some(TokenTree::Group(block)) => block.stream(),
                _ => fail!("expected block")
            };
            r.into()
        }
    }
}
//Some(Group { delimiter: Brace, stream: TokenStream [Ident { ident: "A", span: #0 bytes(756..757) }, Punct { ch: ',', spacing: Alone, span: #0 bytes(757..758) }, Ident { ident: "B", span: #0 bytes(76 7..768) }, Group { delimiter: Brace, stream: TokenStream [Ident { ident: "x", span: #0 bytes(783..784) }, Punct { ch: ':', spacing: Alone, span: #0 bytes(784..785) }, Punct { ch: '&', spacing: Alone , span: #0 bytes(786..787) }, Ident { ident: "str", span: #0 bytes(787..790) }, Punct { ch: ',', spacing: Alone, span: #0 bytes(790..791) }], span: #0 bytes(769..801) }, Punct { ch: ',', spacing: Al one, span: #0 bytes(801..802) }], span: #0 bytes(746..808) }) unexpected Punct { ch: ',', spacing: Alone, span: #0 bytes(808..809) }


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
   
    let mut messages_block = quote!{};
    let mut extra_state_block = quote!{};
    
    while let Some(e) = input.next() {
        match e {
            TokenTree::Ident(i) => {
                match i.to_string().as_str() {
                    "Options" => parse_options!(opts, input),
                    "Messages" => { messages_block = parse_block!(input); },
                    "State" => { extra_state_block = parse_block!(input); },
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
        state_init_opts_block = quote! { #state_init_opts_block , sup_ch: sup_ch, };
    };

    if let Some(t) = opts.idtype {
      let bt = format_ident!("{}", t);
        start_params = quote! { #start_params , id:  &#bt };
        state_opts_block = quote! { #state_opts_block id: #bt, };
        state_init_opts_block = quote! { #state_init_opts_block  id: id.clone(), };
       
    };
 
    quote!{
        impl Default for mpsc::Sender<#messages_ident> {
            fn default() -> mpsc::Sender<BarActorMessages> {
                let (tx, _) = mpsc::channel(1);
                tx
            }
        }

        #preamble

        #[derive(Debug)]
        enum #messages_ident {
            #messages_block
        }

        #[derive(Debug, Default)]
        struct #state_ident {
            rx: mpsc::Receiver<#messages_ident>,
            tx: mpsc::Sender<#messages_ident>,
            #state_opts_block
            #extra_state_block 
        }

        impl #actor_ident {
            fn start(mut self #start_params) -> mpsc::Sender<#messages_ident> {
                let (tx, rx) = mpsc::channel(100); 
                let mut state = #state_ident {
                    rx: rx,
                    tx: tx.clone(),
                    #state_init_opts_block
                    #extra_state_block
                };
                tokio::spawn(async move {
                    while let Some(msg) = state.rx.recv().await {
                        self.handle(&mut state, msg);
                    }
                });
                tx 
            }
        }
    }.into()

}


