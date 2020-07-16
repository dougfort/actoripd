use actix::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
enum Action {
    COOPERATE,
    DEFECT,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
enum Payoff {
    /// Start the interrogation
    NULL,

    /// If both players cooperate, they both receive the reward R for cooperating.
    REWARD,

    /// If both players defect, they both receive the punishment Payoff P.
    PUNISHMENT,

    /// If Blue defects while Red cooperates, then Blue receives the temptation Payoff T, while Red receives the "sucker's" Payoff, S.
    /// Similarly, if Blue cooperates while Red defects, then Blue receives the sucker's Payoff S, while Red receives the temptation Payoff T.
    TEMPTATION,
    SUCKER,
}

type PayoffValues = HashMap<Payoff, usize>;

struct Interrogator {
    iterations: usize,
    sequence: usize,
    blue_addr: Addr<Prisoner>,
    blue_payoff: Payoff,
    blue_amount: usize,
    red_addr: Addr<Prisoner>,
    red_payoff: Payoff,
    red_amount: usize,
}

impl Actor for Interrogator {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.interrogate(ctx);
    }
}

impl Interrogator {
    fn interrogate(&mut self, ctx: &mut Context<Self>) {
        let sequence = self.sequence;
        self.sequence += 1;
            
        let prev_payoff = self.blue_payoff;
        let prev_amount = self.blue_amount;
        self.blue_addr.do_send(
            Interrogate{
                sequence,
                prev_payoff,
                prev_amount,
            }
        );

        let prev_payoff = self.red_payoff;
        let prev_amount = self.red_amount;
        self.red_addr.do_send(
            Interrogate{
                sequence,
                prev_payoff,
                prev_amount,
            }
        );
    }
}

struct Interrogate {
    sequence: usize,
    prev_payoff: Payoff,
    prev_amount: usize,
}

impl Message for Interrogate {
    type Result = Action;
}

impl Handler<Interrogate> for Prisoner {
    type Result = MessageResult<Interrogate>;

    fn handle(&mut self, msg: Interrogate, ctx: &mut Context<Self>) -> Self::Result {
        println!("{}: Interrogate received: sequence = {}", self.name, msg.sequence);

        // TODO: use a strategy to pick an Action
        MessageResult(Action::DEFECT)
    }
}

struct Prisoner {
    name: String,
}

impl Actor for Prisoner {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "Action")]
struct Deal {
    iteration: usize,
}


fn main() {
    const ITERATIONS: usize = 10;
    let mut Payoff_values: PayoffValues = HashMap::new();
    Payoff_values.insert(Payoff::REWARD, 3);
    Payoff_values.insert(Payoff::TEMPTATION, 4);
    Payoff_values.insert(Payoff::PUNISHMENT, 2);
    Payoff_values.insert(Payoff::SUCKER, 1);

    let system = System::new("prisoners-dilemma");

    let execution = async {
        let blue_addr = Prisoner{name: "blue".to_owned()}.start();
        let red_addr = Prisoner{name: "red".to_owned()}.start();
        let _interrogator_addr = Interrogator{
            iterations: ITERATIONS,
            sequence: 0,
            blue_addr,
            blue_payoff: Payoff::NULL,
            blue_amount: 0,
            red_addr,
            red_payoff: Payoff::NULL,
            red_amount: 0,
        }
        .start();

        println!("Hello, world!");
    };
    Arbiter::spawn(execution);

    System::current().stop();

    system.run().unwrap();
}
