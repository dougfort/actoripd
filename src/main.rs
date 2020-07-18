use actix::prelude::*;
use log::debug;
use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;
use std::fmt;

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
        self.score += msg.prev_amount;
        let action = self.strategy.choose();

        debug!(
            "{}: Interrogate received: sequence = {}; prev payoff = {}, prev amount = {}, score = {} => action = {}",
            self.name, msg.sequence, msg.prev_payoff, msg.prev_amount, self.score, action
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
    score: usize,
}

impl Actor for Prisoner {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        debug!("Actor {}: starts", self.name);
    }
 
    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        debug!("Actor {}: stops: final score: {}", self.name, self.score);
    }
 }

fn main() {
    const ITERATIONS: usize = 100;

    std::env::set_var("RUST_LOG", "actoripd=debug,actix=info");
    env_logger::init();

    let system = System::new("prisoners-dilemma");

    let execution = async {
        let mut payoff_values: PayoffValues = HashMap::new();
        payoff_values.insert(Payoff::REWARD, 3);
        payoff_values.insert(Payoff::TEMPTATION, 4);
        payoff_values.insert(Payoff::PUNISHMENT, 2);
        payoff_values.insert(Payoff::SUCKER, 1);

        let blue_addr = Prisoner {
            name: "blue".to_owned(),
            strategy: Box::new(RandomStrategy {}),
            score: 0,
        }
        .start();
        let red_addr = Prisoner {
            name: "red".to_owned(),
            strategy: Box::new(RandomStrategy {}),
            score: 0,
        }
        .start();

        let mut sequence = 0;
        let mut blue_payoff = Payoff::NULL;
        let mut blue_amount = 0;
        let mut red_payoff = Payoff::NULL;
        let mut red_amount = 0;

        loop {
            let blue_result = blue_addr
                .send(Interrogate {
                    sequence,
                    prev_payoff: blue_payoff,
                    prev_amount: blue_amount,
                })
                .await;

            let red_result = red_addr
                .send(Interrogate {
                    sequence,
                    prev_payoff: red_payoff,
                    prev_amount: red_amount,
                })
                .await;

            let red_action = red_result.unwrap();
            let blue_action = blue_result.unwrap();

            let payoff = compute_payoff(red_action, blue_action);

            red_payoff = payoff.0;
            red_amount = *payoff_values.get(&red_payoff).unwrap_or(&0);

            blue_payoff = payoff.1;
            blue_amount = *payoff_values.get(&blue_payoff).unwrap_or(&0);

            sequence += 1;
            if sequence >= ITERATIONS {
                debug!("completed {} iterations", sequence);
                break;
            }
        }

        System::current().stop();

    };
    Arbiter::spawn(execution);

    system.run().unwrap();
}

impl Strategy for Action {
    fn choose(&mut self) -> Action {
        *self
    }
}

struct RandomStrategy {}

impl Strategy for RandomStrategy {
    fn choose(&mut self) -> Action {
        let action_number = thread_rng().gen::<u8>();
        if action_number % 2 == 0 {
            Action::COOPERATE
        } else {
            Action::DEFECT
        }
    }
}

/// For payoff https://en.wikipedia.org/wiki/Prisoner's_dilemma
///
/// If both players cooperate, they both receive the reward R for cooperating.
/// If both players defect, they both receive the punishment payoff P.
/// If Blue defects while Red cooperates, then Blue receives the temptation payoff T, while Red receives the "sucker's" payoff, S.
/// Similarly, if Blue cooperates while Red defects, then Blue receives the sucker's payoff S, while Red receives the temptation payoff T.
///
/// T > R > P > S
/// We want 2R > T + S for the iterative game
///

fn compute_payoff(red: Action, blue: Action) -> (Payoff, Payoff) {
    match (red, blue) {
        (Action::COOPERATE, Action::COOPERATE) => (Payoff::REWARD, Payoff::REWARD),
        (Action::DEFECT, Action::DEFECT) => (Payoff::PUNISHMENT, Payoff::PUNISHMENT),
        (Action::DEFECT, Action::COOPERATE) => (Payoff::SUCKER, Payoff::TEMPTATION),
        (Action::COOPERATE, Action::DEFECT) => (Payoff::TEMPTATION, Payoff::SUCKER),
    }
}
