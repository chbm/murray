# Murray

The `murray` crate provides a `actor!` macro that helps defining erlang inspired actors. The actor is targeted at async and tokio but can be adapted by `use`ing other `mpsc` channels.

## Use

```
struct MyMsg {
 s: String
 }

actor! {
    Foo,
    Messages: {
        Msg1,
        Msg2 MyMsg,
        Msg3 { ch: mpsc::Receiver<String> }
    }
}

actor! {
    Bar,
    Options: {
        sup: Foo,
        id: String,
    },
    Messages: {
        A,
        B {
            x: bool,
        },
    },
    State: {
        foo: TypeC,
    }
}

impl FooActor {
    async fn handle_msg1(&self, state: &mut FooActorState)  {
	...
    }
    async fn handle_msg2(&self, state: &mut FooActorState, msg: MyMsg)  {
	...
    }
    async fn handle_msg3(&self, state: &mut FooActorState, msg: FooActorMessagesMsg3)  {
	...
    }
}

impl BarActor {
    async fn handle_a(&self, state: &mut BarActorState)  {
	...
    }
    async fn handle_b(&self, state: &mut BarActorState, msg: BarActorMessagesB)  {
	...
    }
    fn init(&self, state: &mut BarActorState) {
	...
    }
}

let sup = FooActor{}.start();
let id = String::from("abar");
let abar = BarActor{}.start(sup, &id);

abar.send(BarActorMessages::B(true));

```

This will produce `struct FooActor`, `enum FooActorMessages` and a `struct FooActorState` (and similar for Bar). 
If you include `Options` they may include a `sup` property naming the agent's supervisor base name and a `id` naming the type of actors id. The type must be Clone.

The State struct includes a `tx` `Sender` channel so that your handlers can send messages back to the actor. If the actor has a supervisor it will also include a `sup_ch` and an `id` field if it's included in options. The actor definition includes a `State` with extra properties they will be included in the state struct as `Option` initialized to None. If you have extra properties `actor::start` will invoke `self.init` before starting processing messages.

The macro expands message variants with properties into corresponding `struct`s with the properties for easier handling in handler functions. So for `Foo` the macro generates a `struct FooActorMessagesMsg3` but no `struct FooActorMessagesMsg1` or  `struct FooActorMessagesMsg2` and expects you to provide `FooActor::handle_msg1`, `FooActor::handle_msg2` and `FooActor::handle_msg3`. The handler functions are `async` and return `()`. All communication with the actor is done via the `state.tx` channel. Remeber, your handlers can't take ownership of the channel and you need to move a `clone`.

For more examples see the murray-tests repo.

## Caveats
`FooActorMessagesMsg1` is a terrible identifier and the macro should provide a shorter version.

There's not actual supervision yet, but otoh there's no actor isolation so a crash would take everything down. 





