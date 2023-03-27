
use std::{io::{BufReader, BufRead}, fs::File, collections::{VecDeque, HashMap, HashSet}, num};
use regex::Regex;

const MaxTime: u16 = 32;

#[derive(Debug, Clone, PartialEq)]
enum Decision {
    BuildOreRobot,
    BuildClayRobot,
    BuildObsidianRobot,
    BuildGeodeRobot
}


#[derive(Debug)]
struct Blueprint {
    cost_ore_robot: u16,
    cost_clay_robot: u16,
    cost_obsidian_robot: (u16, u16),
    cost_geode_robot: (u16, u16)
}

#[derive(Debug, Clone)]
struct StockPile {
    ore: u16,
    clay: u16,
    obsidian: u16,
    geode: u16
}

#[derive(Debug, Clone)]
struct WorkForce {
    ore_robots: u16,
    clay_robots: u16,
    obsidian_robots: u16,
    geode_robots: u16
}

#[derive(Debug, Clone)]
struct WalkState {
    time: u16,
    stockpile: StockPile,
    workforce: WorkForce
}

fn simulate(blueprint: &Blueprint) -> u16 {

    let inffeasiblity_filter = InfeasibilityFilter::new(200);

    let stockpile = StockPile{ore: 0, clay: 0, obsidian: 0, geode: 0};
    let workforce = WorkForce{ore_robots: 1, clay_robots: 0, obsidian_robots: 0, geode_robots: 0};
    let time: u16 = 0;

    let init_state = WalkState { time: time, stockpile, workforce };
    let initial_options = calculate_next_decision_point(&init_state, &blueprint);

    let mut Q: VecDeque<(WalkState, u16, Decision)>  = VecDeque::new();

    for o in initial_options {
        Q.push_back((init_state.clone(), o.0, o.1));
    }
    
    let mut best_result = 0;

    while !Q.is_empty() {

        let (previous_state, time_delta, decission) = Q.pop_front().unwrap();

        let current_state = tick(previous_state, time_delta, decission, blueprint);

        if inffeasiblity_filter.purge_infeasible(&current_state, best_result) {
            continue;
        }

        let options = calculate_next_decision_point(&current_state, &blueprint);

        for o in options {

            if current_state.time + o.0 >= MaxTime {
                let result_candidate = current_state.stockpile.geode + (MaxTime - current_state.time) * current_state.workforce.geode_robots;

                if result_candidate > best_result {
                    best_result = result_candidate;
                    println!("Found better solution: {}", best_result);
                }
            } else {
                Q.push_back((current_state.clone(), o.0, o.1));
            }
        }
    }

    best_result
}

fn tick(previous_state: WalkState, time_delta: u16, decission: Decision, blueprint: &Blueprint) -> WalkState {

    let mut current_state = previous_state.clone();

    current_state.stockpile.ore += current_state.workforce.ore_robots * time_delta;
    current_state.stockpile.clay += current_state.workforce.clay_robots * time_delta;
    current_state.stockpile.obsidian += current_state.workforce.obsidian_robots * time_delta;
    current_state.stockpile.geode += current_state.workforce.geode_robots * time_delta;
    
    current_state.time += time_delta;

    match decission {
        Decision::BuildOreRobot => {
            current_state.stockpile.ore += current_state.workforce.ore_robots;
            current_state.stockpile.clay += current_state.workforce.clay_robots;
            current_state.stockpile.obsidian += current_state.workforce.obsidian_robots;
            current_state.stockpile.geode += current_state.workforce.geode_robots;

            current_state.time += 1;

            current_state.stockpile.ore -= blueprint.cost_ore_robot;

            current_state.workforce.ore_robots += 1;
        },
        Decision::BuildClayRobot => {
            current_state.stockpile.ore += current_state.workforce.ore_robots;
            current_state.stockpile.clay += current_state.workforce.clay_robots;
            current_state.stockpile.obsidian += current_state.workforce.obsidian_robots;
            current_state.stockpile.geode += current_state.workforce.geode_robots;

            current_state.time += 1;

            current_state.stockpile.ore -= blueprint.cost_clay_robot;

            current_state.workforce.clay_robots += 1;
        },
        Decision::BuildObsidianRobot => {
            current_state.stockpile.ore += current_state.workforce.ore_robots;
            current_state.stockpile.clay += current_state.workforce.clay_robots;
            current_state.stockpile.obsidian += current_state.workforce.obsidian_robots;
            current_state.stockpile.geode += current_state.workforce.geode_robots;

            current_state.time += 1;

            current_state.stockpile.ore -= blueprint.cost_obsidian_robot.0;
            current_state.stockpile.clay -= blueprint.cost_obsidian_robot.1;

            current_state.workforce.obsidian_robots += 1;
        },
        Decision::BuildGeodeRobot => {
            current_state.stockpile.ore += current_state.workforce.ore_robots;
            current_state.stockpile.clay += current_state.workforce.clay_robots;
            current_state.stockpile.obsidian += current_state.workforce.obsidian_robots;
            current_state.stockpile.geode += current_state.workforce.geode_robots;

            current_state.time += 1;

            current_state.stockpile.ore -= blueprint.cost_geode_robot.0;
            current_state.stockpile.obsidian -= blueprint.cost_geode_robot.1;

            current_state.workforce.geode_robots += 1;
        }
    }

    return current_state;

}

fn calculate_next_decision_point(walkstate: &WalkState, blueprint: &Blueprint) -> Vec<(u16, Decision)> {

    let mut decissions = Vec::new();
    let mut time_delta = 0;
    let mut novel_decissions = Vec::new();

    if walkstate.stockpile.ore < blueprint.cost_ore_robot {

        let mut f = (blueprint.cost_ore_robot - walkstate.stockpile.ore) / walkstate.workforce.ore_robots;
        f += ((blueprint.cost_ore_robot - walkstate.stockpile.ore) % walkstate.workforce.ore_robots > 0) as u16;

        time_delta = f;
        novel_decissions.push((f, Decision::BuildOreRobot));
    } else {
        decissions.push((0, Decision::BuildOreRobot));
    }

    if walkstate.stockpile.ore < blueprint.cost_clay_robot {

        let mut f = (blueprint.cost_clay_robot - walkstate.stockpile.ore) / walkstate.workforce.ore_robots;
        f += ((blueprint.cost_clay_robot - walkstate.stockpile.ore) % walkstate.workforce.ore_robots > 0) as u16;

        novel_decissions.push((f, Decision::BuildClayRobot));

    } else {
        decissions.push((0, Decision::BuildClayRobot));
    }

    if walkstate.workforce.clay_robots > 0 {
        if walkstate.stockpile.ore < blueprint.cost_obsidian_robot.0 || walkstate.stockpile.clay < blueprint.cost_obsidian_robot.1 {

            let mut f = 0;
            let mut g = 0;
    
            if walkstate.stockpile.ore < blueprint.cost_obsidian_robot.0 {
                f = (blueprint.cost_obsidian_robot.0 - walkstate.stockpile.ore) / walkstate.workforce.ore_robots;
                f += ((blueprint.cost_obsidian_robot.0 - walkstate.stockpile.ore) % walkstate.workforce.ore_robots > 0) as u16;
            }
    
            if walkstate.stockpile.clay < blueprint.cost_obsidian_robot.1 {
                g = (blueprint.cost_obsidian_robot.1 - walkstate.stockpile.clay) / walkstate.workforce.clay_robots;
                g += ((blueprint.cost_obsidian_robot.1 - walkstate.stockpile.clay) % walkstate.workforce.clay_robots > 0) as u16;
            }
    
            let h = std::cmp::max(f,g);

            novel_decissions.push((h, Decision::BuildObsidianRobot));
        } else {
            decissions.push((0, Decision::BuildObsidianRobot));
        }
    } 

    if walkstate.workforce.obsidian_robots > 0 {
        if walkstate.stockpile.ore < blueprint.cost_geode_robot.0 || walkstate.stockpile.obsidian < blueprint.cost_geode_robot.1 {

            let mut f = 0;
            let mut g = 0;
    
            if walkstate.stockpile.ore < blueprint.cost_geode_robot.0 {
                f = (blueprint.cost_geode_robot.0 - walkstate.stockpile.ore) / walkstate.workforce.ore_robots;
                f += ((blueprint.cost_geode_robot.0 - walkstate.stockpile.ore) % walkstate.workforce.ore_robots > 0) as u16;
            }
    
            if walkstate.stockpile.obsidian < blueprint.cost_geode_robot.1 {
                g = (blueprint.cost_geode_robot.1 - walkstate.stockpile.obsidian) / walkstate.workforce.obsidian_robots;
                g += ((blueprint.cost_geode_robot.1 - walkstate.stockpile.obsidian) % walkstate.workforce.obsidian_robots > 0) as u16;
            }
    
            let h = std::cmp::max(f,g);
    
            novel_decissions.push((h, Decision::BuildGeodeRobot));
        } else {
            decissions.push((0, Decision::BuildGeodeRobot));
        }
    } 

    for n in novel_decissions {
        decissions.push(n);
    }

    return decissions;
}

struct InfeasibilityFilter {

    geode_series: Vec<(u16, u16)>
}

impl InfeasibilityFilter {

    fn new(size: u16) -> Self {

        let mut series = vec![];
        let mut prev = (0, 0);

        for i in 0..size {

            let curr = (i, prev.0 + prev.1);
            prev = curr;
            series.push(curr);

        }

        Self{geode_series: series}
    }   

    fn purge_infeasible(self: &Self, walkstate: &WalkState, best_candidate: u16) -> bool {

        let geodic_robots = walkstate.workforce.geode_robots;
        let geode = walkstate.stockpile.geode;

        let start_series = self.geode_series[geodic_robots as usize];
        let end_series = self.geode_series[geodic_robots as usize + (MaxTime - walkstate.time) as usize];

        let lower_bound = end_series.1 - start_series.1 + geode;

        best_candidate > lower_bound
    }
}



fn main() {

    let reader = BufReader::new(File::open("input.txt").unwrap());

    let re = Regex::new(r"Blueprint (\d+): Each ore robot costs (\d+) ore. Each clay robot costs (\d+) ore. Each obsidian robot costs (\d+) ore and (\d+) clay. Each geode robot costs (\d+) ore and (\d+) obsidian.").unwrap();

    let mut blueprints = vec![];

    for line in reader.lines().map(|l| l.unwrap()) {

        let numbers: Vec<u16> = re.captures_iter(
            line.as_str()).next().unwrap().iter().map(
                |cap| cap.unwrap().parse::<u16>().unwrap_or(0)).collect();

        let b = Blueprint{
            cost_ore_robot: numbers[2],
            cost_clay_robot: numbers[3],
            cost_obsidian_robot: (numbers[4], numbers[5]),
            cost_geode_robot: (numbers[6], numbers[7])};
        
        blueprints.push(b);
    }

    let mut res = 1;

    for (i, blueprint) in blueprints[0..3].iter().enumerate() {
        println!("blueprint {} {:?}", i, blueprint);

        res *= simulate(blueprint) as usize;
    }

    println!("final quantity {}", res);
   
}