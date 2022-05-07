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

impl FooActor {
    fn handle(&self, state: &mut FooActorState, msg: FooActorMessages) -> () {
        ()
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

impl BarActor {
    fn handle(&self, state: &mut BarActorState, msg: BarActorMessages) -> () {
        ()
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





