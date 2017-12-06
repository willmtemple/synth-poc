extern crate rayon;

use rayon::prelude::*;

use std::collections::{HashSet, HashMap};
use std::io;
use std::io::prelude::*;
use std::num::Wrapping;

macro_rules! vectuple(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut v = vec![];
            $(
                v.push(($key, $value));
            )+
            v
        }
    };
);

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Vr(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Instr {
    Mov(Vr, Vr),
    Add(Vr, Vr),
    Mul(Vr, Vr),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Program {
    code: Vec<Instr>,
    vrs: Vec<Vr>,
    next_vr: usize,
    input_vrs: Vec<Vr>,
}

impl Program {
    pub fn new(num_inputs: usize) -> Program {
        let mut p = Program {
            code: vec![],
            vrs: vec![],
            next_vr: 0,
            input_vrs: vec![],
        };

        for _ in 0..num_inputs {
            p.make_input_vr();
        }

        p
    }

    pub fn len(&self) -> (usize, usize) {
        (self.code.len(), self.vrs.len())
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
    fn simulate(&self, inputs: Vec<i32>, output: bool) -> HashMap<Vr, i32>;
}

impl Simulate for Program {
    fn simulate(&self, inputs: Vec<i32>, output: bool) -> HashMap<Vr, i32> {

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
                        None => panic!("Required register {:?} has no value.", from),
                    };
                    map.insert(to, v_required);
                },
                Mul(ref from, ref to) => {
                    let f_required = match map.get(from) {
                        Some(v) => Wrapping(*v),
                        None => panic!("Required register {:?} has no value.", from),
                    };
                    let t_required = match map.get(to) {
                        Some(v) => Wrapping(*v),
                        None => panic!("Required register {:?} has no value.", from),
                    };
                    map.insert(*to, (f_required * t_required).0);
                },
                Add(ref from, ref to) => {
                    let f_required = match map.get(from) {
                        Some(v) => Wrapping(*v),
                        None => panic!("Required register {:?} has no value.", from),
                    };
                    let t_required = match map.get(to) {
                        Some(v) => Wrapping(*v),
                        None => panic!("Required register {:?} has no value.", from),
                    };
                    map.insert(*to, (f_required + t_required).0);
                },
                _ => unimplemented!(),
            }
            if output { println!("{:?}", &map); }
        }

        map
    }
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VerificationResult {
    Correct(Vr), // Indicates that the given Vr contains the correct output
    LinearNear(Vr, i32, i32), // Returns a linear adjustment LinearNear(vrx, m, b) indicating that vrx should be adjusted by vrx = vrx * m + b
    Invalid,
}

fn verify(all_vrs: HashSet<Vr>, tests: Vec<(HashMap<Vr, i32>, i32)>) -> VerificationResult {
    use VerificationResult::*;

    let mut good_vrs = all_vrs.clone();
    for (ass, result) in tests {
        good_vrs = good_vrs
            .intersection(&contained_in(ass, result))
            .map(|r| *r)
            .collect();
    }

    if good_vrs.is_empty() {
        Invalid
    } else {
        Correct(match good_vrs.iter().nth(0) {
            Some(r) => *r,
            None => panic!("No first item on nonempty iter :/"),
        })
    }
}

fn as_output_set(full_output: Vec<(HashMap<Vr, i32>, i32)>) -> Vec<Vec<i32>> {
    let mut r = vec![];
    for (m, _) in full_output {
        let mut ivec = vec![];
        for u in 0..m.keys().len() {
            ivec.push(*m.get(&Vr(u)).unwrap());
        }
        r.push(ivec);
    }
    //println!("Constructed output set {:?}", r);
    r
}

fn main() {

    /*let insize = 1; // f(x) = x**2
    let testinputs: HashMap<Vec<i32>, i32> =
        map!{
        vec![20] => 400,
        vec![1] => 1,
        vec![-1] => 1,
        vec![-20] => 400,
        vec![0] => 0,
        vec![400] => 160000
    };*/

    /*let insize = 2; // f(x,y) = (x+y)**2
    let testinputs = vectuple!{
        vec![0,1] => 1,
        vec![0,2] => 4,
        vec![0,4] => 16,
        vec![-1,1] => 0,
        vec![0,0] => 0
    };*/

    let insize = 2; // f(x,y,z) = (x * y + z**2)**2 * z, generated with real python
    let testinputs =
        vectuple!{
        vec![0,0] => 0,
vec![0,1] => 1,
vec![0,-1] => 1,
vec![1,1] => 5,
vec![1,-1] => -1,
vec![-1,1] => -1,
vec![-1,-1] => 5
    };

    /*let insize = 3;
    let testinputs = vectuple!{
        vec![0,0,0] => 0,
vec![0,0,1] => 1,
vec![0,0,-1] => 1,
vec![0,1,0] => 1,
vec![0,1,1] => 8,
vec![0,1,-1] => 8,
vec![0,-1,0] => -1,
vec![0,-1,1] => 0,
vec![0,-1,-1] => 0,
vec![1,0,0] => 1,
vec![1,0,1] => 8,
vec![1,0,-1] => 8,
vec![1,1,0] => 9,
vec![1,1,1] => 28,
vec![1,1,-1] => 28,
vec![1,-1,0] => -1,
vec![1,-1,1] => 0,
vec![1,-1,-1] => 0,
vec![-1,0,0] => -1,
vec![-1,0,1] => 0,
vec![-1,0,-1] => 0,
vec![-1,1,0] => -1,
vec![-1,1,1] => 0,
vec![-1,1,-1] => 0,
vec![-1,-1,0] => -7,
vec![-1,-1,1] => 0,
vec![-1,-1,-1] => 0
    };*/
    let mut p = Program::new(insize);

    let mut programs: HashSet<Program> = HashSet::new();
    let mut visited = HashSet::new();

    programs.insert(p.clone());

    let mut testresults = vec![];
    for (tin, tout) in testinputs.clone() {
        testresults.push((p.clone().simulate(tin, false), tout));
    }
    visited.insert(as_output_set(testresults.clone()));

    println!(
        "The null program {:?} is {:?}",
        p,
        verify(p.vr_set(), testresults)
    );

    let mut generation = 0;
    loop {
        println!("Generation {}", generation);
        generation += 1;

        let oprogs = programs.clone();
        let nprogs = oprogs.par_iter().flat_map(|p| p.derive());

        let results = nprogs
            .map(|p| {
                    let outputs =
                        testinputs
                            .par_iter()
                            .map(|i| (p.simulate(i.0.clone(), false), i.1))
                            .collect::<Vec<(HashMap<Vr,i32>,i32)>>();
                    (p.clone(), as_output_set(outputs.clone()), verify(p.vr_set(), outputs))
                })
            .filter(|rt| !visited.contains(&rt.1))
            .collect::<Vec<(Program, Vec<Vec<i32>>, VerificationResult)>>();

        let cresults = results.clone();
        let have_good_program = cresults.par_iter().find_any(|rt| match rt.2 {
            VerificationResult::Invalid => false,
            _ => true,
        });

        match have_good_program {
            Some(&(ref p, _, vres)) => {
                println!("Found good program: {:?}, result in {:?}", p, vres);
                return;
            },
            None => {},
        }

        programs = results.par_iter().map(|rt| rt.0.clone()).collect::<HashSet<Program>>();

        /*
        let old_progs = programs.clone();
        programs = HashSet::new();
        for partial in old_progs {
            for dp in partial.derive() {
                let instrs = dp.len().0;
                //println!("l = {:?}", dp.len());
                let mut results = vec![];
                for (tin, tout) in testinputs.clone() {
                    match dp.simulate(tin, false) {
                        Some(s) => results.push((s, tout)),
                        None => unimplemented!()
                    };
                }

                /*if dp.len() == (6,4) {
                    println!("{:?} -> {:?}", dp.clone(), results.clone());
                    pause();
                }*/

                let valid = verify(dp.vr_set(), results.clone());
                match valid {
                    VerificationResult::Correct(cvr) => {
                        println!(
                            "Program {:?} has been verified, the result is in {:?}",
                            dp,
                            cvr
                        );
                        return;
                    }
                    VerificationResult::LinearNear(nvr, m, b) => {
                        unimplemented!();
                    }
                    VerificationResult::Invalid => {
                        let r = as_output_set(results);
                        if visited.contains(&r) {
                            //println!("discarding {:?} as equivalent to a prior partial", dp);
                        } else {
                            //println!("admitting {:?} as unique", dp);
                            visited.insert(r);
                            programs.insert(dp);
                        }

                    }
                }
            }
        }*/
    }
}
