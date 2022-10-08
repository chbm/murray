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

## Actors ? 

The Actor Model is a model of concurrent computation in which "actors" are self contained units of compute with totally encapsulated state. The world outside the actor comunicates with the actor only by sending it messages. In response to the messages the actor can change its state, send messages and spawn more actors. Conceptually it's similar to Communicating Sequencial Programms, the compute model the Golang runtime implements. 

The actor model has three features that make it increasingly relevant

 * complete "object" encapsulation
 * shared nothing
 * lockless concurrency 

No external interference into actor states and no shared or global state between actors remove the race type bugs (ex, thread A modifies a shared value without thread B knowledge). This together with actors always being able to advance as they don't depend on external resources (lockless concurrency) allows for writing safe programms that parallelise automatically in current multicore hardware.

Using actors in Rust doesn't make your program bulletproof, specially compared to writing in Erlang. The Erlang VM isolates the actors as if they were unix processes so an error in an actor doesn't affect the others (in fact, the erlang motto is let it crash). Rust is a stack based language and a crashing bug (eg. an `unwrap` panic) will bring down the whole program. 
