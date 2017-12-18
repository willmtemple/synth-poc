#![feature(conservative_impl_trait)]

extern crate typed_arena;
use typed_arena::Arena;

extern crate rayon;
use rayon::prelude::*;

use std::collections::HashSet;

type RegisterIndex = usize;
type Value = isize;

#[derive(Debug)]
enum Instruction {
    Mov(RegisterIndex, RegisterIndex),
    Add(RegisterIndex, RegisterIndex),
    Mul(RegisterIndex, RegisterIndex),
    //Sub(RegisterIndex, RegisterIndex),
    Neg(RegisterIndex),
}

#[derive(Debug)]
struct Program<'a> {
    parent: Option<&'a Execution<'a>>,
    instruction: Option<Instruction>,
}

#[derive(Debug)]
struct Execution<'a> {
    program: Program<'a>,
    output: Vec<Vec<Value>>,
    ordering: Vec<RegisterIndex>,
}

impl<'a> std::cmp::PartialEq for Execution<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.output == other.output
    }
}

impl<'a> std::cmp::Eq for Execution<'a> {}

impl<'a> std::hash::Hash for Execution<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.output.hash(state)
    }
}

impl<'a> std::fmt::Display for Execution<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.program.parent.map_or(Ok(()), |p| p.fmt(f))?;
        match self.program.instruction {
            Some(ref inst) => inst.fmt(f),
            None => Ok(())
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            &Instruction::Mov(r1, r2) => write!(f, "\nmov r{} r{}", r1, r2),
            &Instruction::Add(r1, r2) => write!(f, "\nadd r{} r{}", r1, r2),
            &Instruction::Mul(r1, r2) => write!(f, "\nmul r{} r{}", r1, r2),
            //&Instruction::Sub(r1, r2) => write!(f, "\nsub r{} r{}", r1, r2),
            &Instruction::Neg(r) => write!(f, "\nneg r{}", r),
        }
    }
}

fn add_one_instruction<'a>(parent: &'a Execution) -> Vec<Program<'a>> {
    let parent_register_count = parent.output[0].len();

    let mut new_programs = Vec::with_capacity(
        parent_register_count * 2 + (parent_register_count * parent_register_count * 3),
    );

    // Copy to new register
    for index in 0..parent_register_count {
        new_programs.push(Program {
            parent: Some(parent),
            instruction: Some(Instruction::Mov(index, parent_register_count)),
        });
    }

    // Ops of two existing registers
    for index in 0..parent_register_count {
        for index2 in 0..parent_register_count {
            new_programs.push(Program {
                parent: Some(parent),
                instruction: Some(Instruction::Mov(index, index2)),
            });
            new_programs.push(Program {
                parent: Some(parent),
                instruction: Some(Instruction::Add(index, index2)),
            });
            new_programs.push(Program {
                parent: Some(parent),
                instruction: Some(Instruction::Mul(index, index2)),
            });
            /*new_programs.push(Program {
                parent: Some(parent),
                instruction: Some(Instruction::Sub(index, index2)),
            });*/
        }
        new_programs.push(Program {
            parent: Some(parent),
            instruction: Some(Instruction::Neg(index)),
        })
    }

    debug_assert!(new_programs.len() == new_programs.capacity());
    new_programs
}

fn execute<'a>(program: Program<'a>) -> Execution<'a> {
    let mut all_testcases = program.parent.unwrap().output.clone();

    for mut testcase in &mut all_testcases {
        match program.instruction {
            None => unreachable!(),
            Some(Instruction::Mov(r1, r2)) => if testcase.len() <= r2 {
                let temp = testcase[r1];
                testcase.push(temp);
            } else {
                testcase[r2] = testcase[r1];
            },
            Some(Instruction::Add(r1, r2)) => testcase[r2] = testcase[r1].wrapping_add(testcase[r2]),
            Some(Instruction::Mul(r1, r2)) => testcase[r2] = testcase[r1].wrapping_mul(testcase[r2]),
            //Some(Instruction::Sub(r1, r2)) => testcase[r2] = testcase[r1].wrapping_add(-testcase[r2]),
            Some(Instruction::Neg(r)) => testcase[r] = testcase[r].wrapping_mul(-1),
        };
    }

    Execution {
        program: program,
        output: all_testcases,
        ordering: vec![],
    }
}

fn verify<'a>(exe: &'a Execution<'a>, tests: &Vec<isize>) -> Option<RegisterIndex> {
    let register_count = exe.output[0].len();

    (0..register_count).find(|output_register| {
        exe.output
            .iter()
            .zip(tests.iter())
            .all(|(output, test_value)| {
                output[*output_register] == *test_value
            })
    })
}

macro_rules! testcases [
    ( $( ([ $($input: expr),* ], $output:expr), )* ) => {
        {
        let mut inputs: Vec<Vec<Value>> = Vec::new();
        let mut outputs: Vec<Value> = Vec::new();

        $(
            let mut inputrow = Vec::new();
            $(
                inputrow.push($input);
            )*
            inputs.push(inputrow);
            outputs.push($output);
        )*

        (inputs, outputs)
        }
    }
];

fn main() {
    let (inputs, outputs) = testcases![
        ([0, 0], 0),
        ([0, 1], -1),
        ([1, 1], 0),
        ([1, 0], 1),
        ([0, -1], 1999),
        ([-1, 0], -1),
        ([-1, -1], 0),
        ([1, -1], 2),
        ([-1, 1], -2),       
    ];

    let mut starts = HashSet::default();
    let start_exec = Execution {
        program: Program {
            parent: None,
            instruction: None,
        },
        output: inputs,
        ordering: vec![],
    };

    if verify(&start_exec, &outputs).is_some() {
        println!("Get out.");
        return;
    }

    starts.insert(start_exec);
    let old_executions = Arena::new();
    //let mut prev_generations = Vec::new();
    let mut last_generation = Some(&*old_executions.alloc(starts));
    let mut generation = 1;

    loop {
        println!("{}", generation);
        //prev_generations.push(last_generation.clone().unwrap());

        let old_programs = last_generation.take().unwrap().into_par_iter();

        let new_programs = old_programs.flat_map(add_one_instruction);

        let new_executions = new_programs.map(execute);

        let filtered_executions = new_executions
            // .filter(|newexe| {
            //     prev_generations
            //         .iter()
            //         .all(|prevgen| !prevgen.contains(newexe))
            // })
            .collect::<HashSet<_>>();

        if let Some(exe) = filtered_executions
            .par_iter()
            .find_any(|exe| verify(&exe, &outputs).is_some())
        {
            println!("Found {}", exe);
            println!("Output gets stored in r{}", "?"); // TODO
            break;
        }

        last_generation = Some(old_executions.alloc(filtered_executions));
        generation += 1;
    }
}
