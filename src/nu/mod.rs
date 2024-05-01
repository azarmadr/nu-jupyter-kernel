use std::fmt::{Debug, Display};
use std::{env, mem};

use miette::Diagnostic;
use nu_protocol::debugger::WithoutDebug;
use nu_protocol::engine::{EngineState, FileStack, Stack, StateDelta, StateWorkingSet};
use nu_protocol::{ErrorStyle, ParseError, PipelineData, ShellError, Span, Value};
use thiserror::Error;

pub mod commands;
pub mod render;

pub fn initial_engine_state() -> EngineState {
    // TODO: compare with nu_cli::get_engine_state for other contexts
    let engine_state = nu_cmd_lang::create_default_context();
    let engine_state = nu_command::add_shell_command_context(engine_state);
    let engine_state = nu_cmd_extra::add_extra_command_context(engine_state);
    let engine_state = commands::add_jupyter_command_context(engine_state);
    let engine_state = add_env_context(engine_state);

    engine_state
}

fn add_env_context(mut engine_state: EngineState) -> EngineState {
    if let Ok(current_dir) = env::current_dir() {
        engine_state.add_env_var(nu_protocol::engine::PWD_ENV.to_owned(), Value::String {
            val: current_dir.to_string_lossy().to_string(),
            internal_span: Span::unknown(),
        });
    }

    engine_state
}

#[derive(Error)]
pub enum ExecuteError {
    #[error("{error}")]
    Parse {
        #[source]
        error: ParseError,
        delta: StateDelta,
    },

    #[error(transparent)]
    Shell(#[from] ShellError),
}

impl Debug for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { error, delta } => f
                .debug_struct("Parse")
                .field("error", error)
                .field("delta", &format_args!("[StateDelta]"))
                .finish(),
            Self::Shell(arg0) => f.debug_tuple("Shell").field(arg0).finish(),
        }
    }
}

pub fn execute<'s>(
    code: &str,
    engine_state: &mut EngineState,
    stack: &mut Stack,
    name: &str,
) -> Result<PipelineData, ExecuteError> {
    let code = code.as_bytes();
    let mut working_set = StateWorkingSet::new(engine_state);
    let block = nu_parser::parse(&mut working_set, Some(name), code, false);

    if let Some(error) = working_set.parse_errors.into_iter().next() {
        return Err(ExecuteError::Parse {
            error,
            delta: working_set.delta,
        });
    }

    engine_state.merge_delta(working_set.delta)?;
    let res =
        nu_engine::eval_block::<WithoutDebug>(engine_state, stack, &block, PipelineData::Empty)?;
    Ok(res)
}
