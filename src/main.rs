use std::collections::{HashSet,HashMap};

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
    };
);

#[derive(Debug,Copy,Clone,Eq,PartialEq,Hash)]
pub struct Vr(usize);

#[derive(Debug,Copy,Clone,Eq,PartialEq,Hash)]
pub enum Instr {
    Mov(Vr, Vr),
    Add(Vr, Vr),
    Mul(Vr, Vr)
}

#[derive(Debug,Clone,Eq,PartialEq,Hash)]
pub struct Program {
    code: Vec<Instr>,
    vrs: Vec<Vr>,
    next_vr : usize,
    input_vrs : Vec<Vr>,
}

impl Program {
    pub fn new(num_inputs: usize) -> Program {
        let mut p = Program {
            code: vec![],
            vrs: vec![],
            next_vr: 0,
            input_vrs: vec![],
        };

        for _ in [0..num_inputs].iter() {
            p.make_input_vr();
        }

        p
    }

    pub fn make_vr(&mut self) -> Vr {
        let vr = Vr(self.next_vr);
        self.next_vr += 1;
        self.vrs.push(vr);
        vr
    }

    fn make_input_vr(&mut self) -> Vr {
        let vr = self.make_vr();
        self.input_vrs.push(vr);
        vr
    }

    fn vr_set(&self) -> HashSet<Vr> {
        let mut r = HashSet::new();
        for vr in self.vrs.clone() {
            r.insert(vr);
        }
        r
    }

    pub fn add_instr(&mut self, instr: Instr) {
        self.code.push(instr);
    }
}

pub trait Derive {
    fn derive(&self) -> HashSet<Program>;
}

impl Derive for Program {
    fn derive(&self) -> HashSet<Program> {
        let mut d = HashSet::new();
        // Aliases
        for vr in self.vrs.clone() {
            let mut newprog = self.clone();
            let nvr = newprog.make_vr();
            newprog.add_instr(Instr::Mov(vr, nvr));
            d.insert(newprog);
        }

        // Operations
        
        //Mul
        for vrf in self.vrs.clone() {
            for vrt in self.vrs.clone() {
                let mut newprog = self.clone();
                newprog.add_instr(Instr::Mul(vrf, vrt));
                d.insert(newprog);
            }
        }

        //Add
        for vrf in self.vrs.clone() {
            for vrt in self.vrs.clone() {
                let mut newprog = self.clone();
                newprog.add_instr(Instr::Add(vrf, vrt));
                d.insert(newprog);
            }
        }


        d
    }
}

pub trait Simulate {
    fn simulate(&self, inputs: Vec<i32>) -> HashMap<Vr, i32>;
}

impl Simulate for Program {
    fn simulate(&self, inputs: Vec<i32>) -> HashMap<Vr, i32> {

        use Instr::*;

        assert!(inputs.len() == self.input_vrs.len());

        let mut map = HashMap::new();

        for (i, vr) in self.input_vrs.iter().enumerate() {
            map.insert(*vr, inputs[i]);
        }

        for instr in self.code.clone() {
            match instr {
                Mov(ref from, to) => {
                    let v_required = match map.get(from) {
                        Some(v) => *v,
                        None => panic!("Required register {:?} has no value.", from)
                    };
                    map.insert(to, v_required);
                },
                Mul(ref from, ref to) => {
                    let f_required = match map.get(from) {
                        Some(v) => *v,
                        None => panic!("Required register {:?} has no value.", from)
                    };
                    let t_required = match map.get(to) {
                        Some(v) => *v,
                        None => panic!("Required register {:?} has no value.", from)
                    };
                    map.insert(*to, f_required * t_required);
                },
                Add(ref from, ref to) => {
                    let f_required = match map.get(from) {
                        Some(v) => *v,
                        None => panic!("Required register {:?} has no value.", from)
                    };
                    let t_required = match map.get(to) {
                        Some(v) => *v,
                        None => panic!("Required register {:?} has no value.", from)
                    };
                    map.insert(*to, f_required + t_required);
                },
                _ => unimplemented!(),
            }
        }

        map
    }
}

fn vals_only(assignments: HashMap<Vr, i32>) -> Vec<i32> {
    let mut r = vec![];

    for (_, v) in assignments {
        r.push(v);
    }

    r
}

fn contained_in(assignments: HashMap<Vr, i32>, goal: i32) -> HashSet<Vr> {
    let mut good = HashSet::new();
    for (vr, val) in assignments {
        if val == goal {
            good.insert(vr);
        }
    }
    good
}

#[derive(Debug,Clone,Copy,Eq,PartialEq)]
pub enum VerificationResult {
    Correct(Vr), // Indicates that the given Vr contains the correct output
    LinearNear(Vr, i32, i32), // Returns a linear adjustment LinearNear(vrx, m, b) indicating that vrx should be adjusted by vrx = vrx * m + b
    Invalid,
}

fn verify(all_vrs: HashSet<Vr>, tests: Vec<(HashMap<Vr, i32>,i32)>) -> VerificationResult {
    use VerificationResult::*;

    let mut good_vrs = all_vrs.clone();
    for (ass, result) in tests {
        good_vrs = good_vrs.intersection(&contained_in(ass, result)).map(|r| *r).collect();
    }

    if good_vrs.is_empty() {
        Invalid
    } else {
        Correct(match good_vrs.iter().nth(0) {
            Some(r) => *r,
            None => panic!("No first item on nonempty iter :/")
        })
    }
}

fn main() {

    let testinputs : HashMap<Vec<i32>, i32> = map!{
        vec![20] => 400,
        vec![1] => 1,
        vec![-1] => 1,
        vec![-20] => 400,
        vec![0] => 0,
        vec![400] => 160000
    };
    let mut p = Program::new(1);

    let mut programs : HashSet<Program> = HashSet::new();
    //let mut visited : HashSet<Vec<i32>> = HashSet::new();

    programs.insert(p.clone());

    let mut testresults = vec![];
    for (tin, tout) in testinputs {
        testresults.push((p.clone().simulate(tin),tout));
    }
    //visited.insert(results.clone());

    println!("The null program {:?} is {:?}", p, verify(p.vr_set(), testresults));

    /*loop {
        for partial in programs.clone() {
            programs = HashSet::new();
            for dp in partial.derive() {
                let r = vals_only(dp.simulate(testinput.clone()));
                if visited.contains(&r) {
                    println!("discarding {:?} as equivalent to a prior partial", dp);
                } else {
                    println!("admitting {:?} as unique", dp);
                    visited.insert(r);
                    programs.insert(dp);
                }
            }
        }
    }*/

}
