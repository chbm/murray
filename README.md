# Murray

The `murray` crate provides a `actor` macro that helps defining erlang inspired actors. The actor is targeted at async and tokio but can be adapted by `use`ing other `mpsc` channels.

## Use

```
actor! {
    Foo,
    Messages: {
        Msg1,
        Msg2,
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
    async fn handle_msg2(&self, state: &mut FooActorState)  {
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
}

let sup = FooActor{}.start();
let id = String::from("abar");
let abar = BarActor{}.start(sup, &id);

abar.send(BarActorMessages::B(true));

```

This will produce `struct FooActor`, `enum FooActorMessages` and a `struct FooActorState` (and similar for Bar). 
If you include `Options` they may include a `sup` naming the agent's supervisor and a `id` naming the type of actors id. The type must be Clone.

The State struct includes a `tx` `Sender` channel so that your handlers can send messages back to the actor. If the actor has a supervisor it will also include a `sup_ch` and an `id` field if it's included in options. The actor definition includes a `State` with extra properties they will be included in the state struct as `Option` initialized to None.

The macro expands message variants with propreties into corresponding `struct` with the propreties for easier handling in handler functions. So for `Foo` the macro generates a `struct FooActorMessagesMsg3` but no `struct FooActorMessagesMsg1` or 2 and expects you to provide `FooActor::handle_msg1`, `FooActor::handle_msg2` and `FooActor::handle_msg3`. The handler functions are `async` and return `()`. All communication with the actor is done via `state.tx`.




