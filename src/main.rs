use actix::prelude::*;
use std::collections::HashMap;
use std::fmt;
use log::debug;

#[derive(Clone, Copy)]
enum Action {
    COOPERATE,
    DEFECT,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Action::COOPERATE => "Cooperate",
            Action::DEFECT => "Defect",
        };
        write!(f, "{}", s)
    }
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

impl fmt::Display for Payoff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Payoff::NULL => "Null",
            Payoff::REWARD => "Reward",
            Payoff::PUNISHMENT => "Punishment",
            Payoff::TEMPTATION => "Temptation",
            Payoff::SUCKER => "Sucker",
        };
        write!(f, "{}", s)
    }
}

type PayoffValues = HashMap<Payoff, usize>;

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

    fn handle(&mut self, msg: Interrogate, _ctx: &mut Context<Self>) -> Self::Result {
        let action = self.strategy.choose();

        debug!(
            "{}: Interrogate received: sequence = {}; prev payoff = {}, prev amount = {}, => action = {}",
            self.name, msg.sequence, msg.prev_payoff, msg.prev_amount, action
        );

        MessageResult(action)
    }
}

trait Strategy {
    fn choose(&mut self) -> Action;
}

struct Prisoner {
    strategy: Box<dyn Strategy>,
    name: String,
}

impl Actor for Prisoner {
    type Context = Context<Self>;
}

fn main() {
    const ITERATIONS: usize = 10;
    let mut payoff_values: PayoffValues = HashMap::new();
    payoff_values.insert(Payoff::REWARD, 3);
    payoff_values.insert(Payoff::TEMPTATION, 4);
    payoff_values.insert(Payoff::PUNISHMENT, 2);
    payoff_values.insert(Payoff::SUCKER, 1);

    std::env::set_var("RUST_LOG", "actoripd=debug,actix=info");
    env_logger::init();
    
    let system = System::new("prisoners-dilemma");

    let execution = async {
        let blue_addr = Prisoner {
            name: "blue".to_owned(),
            strategy: Box::new(Action::DEFECT),
        }
        .start();
        let red_addr = Prisoner {
            name: "red".to_owned(),
            strategy: Box::new(Action::COOPERATE),
        }
        .start();

        let mut sequence = 0;
        let blue_payoff = Payoff::NULL;
        let blue_amount = 0;
        let red_payoff = Payoff::NULL;
        let red_amount = 0;
        
        loop {
            let blue_result  = blue_addr.send(Interrogate {
                sequence,
                prev_payoff: blue_payoff,
                prev_amount: blue_amount,
            }).await;

            let red_result = red_addr.send(Interrogate {
                sequence,
                prev_payoff: red_payoff,
                prev_amount: red_amount,
            }).await;

            match red_result {
                Ok(action) => debug!("red action = {}", action),
                Err(err) => debug!("red error = {}", err),
            };

            match blue_result {
                Ok(action) => debug!("blue action = {}", action),
                Err(err) => debug!("blue error = {}", err),
            };

            sequence += 1;
            if sequence >= ITERATIONS {
                debug!("completed {} iterations", sequence);
                break;
            }
        };

    };
    Arbiter::spawn(execution);

    System::current().stop();

    system.run().unwrap();
}

impl Strategy for Action {
    fn choose(&mut self) -> Action {
        *self
    }
}
