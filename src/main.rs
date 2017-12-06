extern crate typed_arena;
extern crate rayon;

use rayon::prelude::*;
use typed_arena::Arena;
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

type Register = usize;

#[derive(Debug)]
enum Instruction {
    Mov(Register, Register),
    Add(Register, Register),
    Mul(Register, Register),
}

#[derive(Debug)]
enum ProgramType<'a> {
    Program(Execution<'a>),
    Start(&'a Vec<(Vec<isize>, isize)>),
}

#[derive(Debug)]
struct Program<'a> {
    parent: &'a ProgramType<'a>,
    instruction: Instruction,
}

#[derive(Debug)]
struct Execution<'a> {
    program: Program<'a>,
    output: Vec<Vec<isize>>,
}

impl<'a> std::cmp::PartialEq for Execution<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.output == other.output
    }
}

impl<'a> std::cmp::Eq for Execution<'a> {}

impl<'a> std::cmp::PartialOrd for Execution<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> std::cmp::Ord for Execution<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.output.cmp(&other.output)
    }
}

impl<'a> std::fmt::Display for Execution<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.program.fmt(f)
    }
}

impl<'a> std::fmt::Display for Program<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.parent.fmt(f)?;
        self.instruction.fmt(f)?;
        Ok(())
    }
}

impl<'a> std::fmt::Display for ProgramType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            &ProgramType::Program(ref exe) => exe.fmt(f),
            &ProgramType::Start(_) => Ok(()),
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            &Instruction::Mov(r1, r2) => write!(f, "mov r{} r{}\n", r1, r2),
            &Instruction::Add(r1, r2) => write!(f, "add r{} r{}\n", r1, r2),
            &Instruction::Mul(r1, r2) => write!(f, "mul r{} r{}\n", r1, r2),
        }
    }
}

fn add_one_instruction<'a>(parent: &'a ProgramType) -> Vec<Program<'a>> {
    let parent_register_count = match parent {
        &ProgramType::Program(ref execution) => execution.output[0].len(),
        &ProgramType::Start(ref tests) => tests[0].0.len(),
    };

    let mut new_programs = Vec::with_capacity(
        parent_register_count + (parent_register_count * parent_register_count * 3),
    );

    // Copy to new register
    for index in 0..parent_register_count {
        new_programs.push(Program {
            parent: parent,
            instruction: Instruction::Mov(index, parent_register_count),
        });
    }

    // Ops of two existing registers
    for index in 0..parent_register_count {
        for index2 in 0..parent_register_count {
            new_programs.push(Program {
                parent: parent,
                instruction: Instruction::Mov(index, index2),
            });
            new_programs.push(Program {
                parent: parent,
                instruction: Instruction::Add(index, index2),
            });
            new_programs.push(Program {
                parent: parent,
                instruction: Instruction::Mul(index, index2),
            });
        }
    }

    new_programs
}

fn execute<'a>(program: Program<'a>) -> Execution<'a> {
    let mut registers = match program.parent {
        &ProgramType::Program(ref execution) => execution.output.clone(),
        &ProgramType::Start(ref tests) => tests.iter().map(|t| t.0.clone()).collect(),
    };

    for state in &mut registers {
        match program.instruction {
            Instruction::Mov(r1, r2) => if state.len() <= r2 {
                let temp = state[r1];
                state.push(temp);
            } else {
                state[r2] = state[r1];
            },
            Instruction::Add(r1, r2) => {
                let i1 = Wrapping(state[r1]);
                let i2 = Wrapping(state[r2]);
                state[r2] = (i1 + i2).0;
            },
            Instruction::Mul(r1, r2) => {
                let i1 = Wrapping(state[r1]);
                let i2 = Wrapping(state[r2]);
                state[r2] = (i1 * i2).0;
            },
        };
    }

    Execution {
        program: program,
        output: registers,
    }
}

fn verify<'a>(
    exe: &'a Execution<'a>,
    tests: &Vec<(Vec<isize>, isize)>,
) -> Option<(&'a Execution<'a>, usize)> {
    let mut testcase = 0;
    let mut correct_registers = exe.output[testcase]
        .par_iter()
        .enumerate()
        .filter(|reg| *reg.1 == tests[testcase].1)
        .map(|reg| reg.0)
        .collect::<Vec<_>>();

    testcase += 1;
    while !correct_registers.is_empty() && testcase < tests.len() {
        let correct_again = exe.output[testcase]
            .par_iter()
            .enumerate()
            .filter(|reg| *reg.1 == tests[testcase].1)
            .map(|reg| reg.0)
            .collect::<Vec<_>>();
        correct_registers.retain(|r| correct_again.contains(r));
        testcase += 1;
    }

    if correct_registers.is_empty() {
        None
    } else {
        Some((exe, correct_registers[0]))
    }
}

fn main() {
    let inputs = vectuple![
        vec![0,0,0] => 0,
vec![0,0,1] => 1,
vec![0,0,-1] => 1,
vec![0,1,0] => 1,
vec![0,1,1] => 8,
vec![0,1,-1] => 8,
vec![0,-1,0] => -1,
vec![1,1,0] => 9,
vec![1,1,1] => 28,
vec![-1,0,0] => -1,
vec![-1,0,1] => 0,
vec![-1,0,-1] => 0,
vec![-1,1,0] => -1,
vec![-1,1,1] => 0,
vec![-1,1,-1] => 0,
vec![-8192,8192,1] => -67108863,
vec![-8192,8192,-1] => -67108863,
vec![-8192,-67108864,8192] => 0,
vec![-8192,-67108864,-8192] => 0,
vec![-67108864,0,8192] => 0,
vec![-67108864,0,-8192] => 0,
vec![-67108864,1,8192] => -67108863,
vec![-67108864,1,-8192] => -67108863,
vec![-67108864,-8192,-8192] => 0
    ];
    let starts = vec![ProgramType::Start(&inputs)];

    let old_code = Arena::new();
    let mut old_generation = Some(old_code.alloc_extend(starts.into_iter()));
    let mut generation = 1;
    loop {
        println!("{:?}", generation);
        let new_programs = old_generation
            .take()
            .unwrap()
            .par_iter()
            .flat_map(|p| add_one_instruction(p));
        let mut new_executions = new_programs.map(execute).collect::<Vec<_>>();
        new_executions.sort_unstable();
        new_executions.dedup();
        if let Some(exe) = new_executions
            .par_iter()
            .filter_map(|exe| verify(&exe, &inputs))
            .find_any(|_| true)
        {
            println!("Found \n{}", exe.0);
            println!("Output gets stored in r{}", exe.1);
            break;
        }
        old_generation = Some(
            old_code.alloc_extend(
                new_executions
                    .into_iter()
                    .map(|exe| ProgramType::Program(exe)),
            ),
        );
        generation += 1;
    }
}
