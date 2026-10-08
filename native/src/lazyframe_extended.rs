//! Extended LazyFrame schema and optimizer operations.

#[cfg(feature = "real-polars")]
use crate::expressions;
use crate::{
    TerlanPolarsDataFrame, TerlanPolarsError, TerlanPolarsFrameSet, TerlanPolarsLazyFrame,
    TerlanPolarsLazyFrameSet,
};

#[cfg(not(feature = "real-polars"))]
fn unavailable() -> TerlanPolarsError {
    TerlanPolarsError::new(
        "polars_unavailable",
        "real Polars support is not enabled for this adapter build",
    )
}

/// Casts named lazy columns with strict or null-producing conversion.
pub fn lazy_cast_columns(
    plan: &TerlanPolarsLazyFrame,
    names: &[String],
    data_types: &[String],
    strict: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, names, data_types, strict);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        if names.len() != data_types.len() {
            return Err(TerlanPolarsError::new(
                "invalid_columns",
                "LazyFrame cast names and data types must have equal lengths",
            ));
        }
        let casts = names
            .iter()
            .zip(data_types)
            .map(|(name, data_type)| {
                let expression = polars::prelude::col(name);
                let data_type = expressions::parse_data_type(data_type)?;
                Ok(if strict {
                    expression.strict_cast(data_type)
                } else {
                    expression.cast(data_type)
                })
            })
            .collect::<Result<Vec<_>, TerlanPolarsError>>()?;
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().with_columns(casts),
        })
    }
}

/// Casts every lazy column to one data type.
pub fn lazy_cast_all(
    plan: &TerlanPolarsLazyFrame,
    data_type: &str,
    strict: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, data_type, strict);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .cast_all(expressions::parse_data_type(data_type)?, strict),
        })
    }
}

/// Returns enabled optimizer flags in stable lexical order.
pub fn lazy_current_optimizations(plan: &TerlanPolarsLazyFrame) -> Vec<String> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        Vec::new()
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::OptFlags;

        let flags = plan.inner.get_current_optimizations();
        [
            ("check_order", OptFlags::CHECK_ORDER_OBSERVE),
            ("cluster_with_columns", OptFlags::CLUSTER_WITH_COLUMNS),
            (
                "common_subexpression_elimination",
                OptFlags::COMM_SUBEXPR_ELIM,
            ),
            ("common_subplan_elimination", OptFlags::COMM_SUBPLAN_ELIM),
            ("eager", OptFlags::EAGER),
            ("gpu", OptFlags::GPU),
            ("predicate_pushdown", OptFlags::PREDICATE_PUSHDOWN),
            ("projection_pushdown", OptFlags::PROJECTION_PUSHDOWN),
            ("row_estimate", OptFlags::ROW_ESTIMATE),
            ("simplify_expression", OptFlags::SIMPLIFY_EXPR),
            ("slice_pushdown", OptFlags::SLICE_PUSHDOWN),
            ("streaming", OptFlags::STREAMING),
            ("type_check", OptFlags::TYPE_CHECK),
            ("type_coercion", OptFlags::TYPE_COERCION),
        ]
        .into_iter()
        .filter(|(_, flag)| flags.contains(*flag))
        .map(|(name, _)| name.to_string())
        .collect()
    }
}

/// Pivots a lazy plan using explicit output-column values.
#[allow(clippy::too_many_arguments)]
pub fn lazy_pivot(
    plan: &TerlanPolarsLazyFrame,
    on_columns: &TerlanPolarsDataFrame,
    on: &[String],
    index: &[String],
    values: &[String],
    aggregation: &str,
    maintain_order: bool,
    separator: &str,
    naming: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (
            plan,
            on_columns,
            on,
            index,
            values,
            aggregation,
            maintain_order,
            separator,
            naming,
        );
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use std::sync::Arc;

        use polars::frame::PivotColumnNaming;
        use polars::prelude::{cols, element};

        if on.is_empty() || index.is_empty() || values.is_empty() {
            return Err(TerlanPolarsError::new(
                "invalid_pivot_columns",
                "lazy pivot requires on, index, and value columns",
            ));
        }
        let aggregate = match aggregation {
            "first" => element().first(),
            "last" => element().last(),
            "sum" => element().sum(),
            "mean" => element().mean(),
            "median" => element().median(),
            "min" => element().min(),
            "max" => element().max(),
            "len" => element().len(),
            other => {
                return Err(TerlanPolarsError::new(
                    "invalid_pivot_aggregation",
                    format!("unsupported lazy pivot aggregation `{other}`"),
                ));
            }
        };
        let naming = match naming {
            "auto" => PivotColumnNaming::Auto,
            "combine" => PivotColumnNaming::Combine,
            other => {
                return Err(TerlanPolarsError::new(
                    "invalid_pivot_naming",
                    format!("unsupported lazy pivot naming strategy `{other}`"),
                ));
            }
        };
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().pivot(
                cols(on.iter().cloned()),
                Arc::new(on_columns.inner.clone()),
                cols(index.iter().cloned()),
                cols(values.iter().cloned()),
                aggregate,
                maintain_order,
                separator.into(),
                naming,
            ),
        })
    }
}

/// Creates an owned lazy-plan set from one plan.
pub fn lazyframe_set_new(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrameSet {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrameSet
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrameSet {
            plans: vec![plan.clone()],
        }
    }
}

/// Returns a derived lazy-plan set with one appended plan.
pub fn lazyframe_set_append(
    set: &TerlanPolarsLazyFrameSet,
    plan: &TerlanPolarsLazyFrame,
) -> TerlanPolarsLazyFrameSet {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (set, plan);
        TerlanPolarsLazyFrameSet
    }
    #[cfg(feature = "real-polars")]
    {
        let mut plans = set.plans.clone();
        plans.push(plan.clone());
        TerlanPolarsLazyFrameSet { plans }
    }
}

/// Returns the number of plans retained by a lazy-plan set.
pub fn lazyframe_set_len(set: &TerlanPolarsLazyFrameSet) -> usize {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = set;
        0
    }
    #[cfg(feature = "real-polars")]
    {
        set.plans.len()
    }
}

/// Adds every plan in a set as an external lazy context.
pub fn lazy_with_context(
    plan: &TerlanPolarsLazyFrame,
    contexts: &TerlanPolarsLazyFrameSet,
) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, contexts);
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        let mut inputs = Vec::with_capacity(contexts.plans.len() + 1);
        inputs.push(plan.inner.clone());
        inputs.extend(contexts.plans.iter().map(|context| context.inner.clone()));
        TerlanPolarsLazyFrame {
            // The input vector always contains the primary plan, satisfying
            // horizontal concatenation's non-empty invariant.
            inner: polars::lazy::dsl::concat_lf_horizontal(inputs, Default::default())
                .expect("with_context always supplies a primary LazyFrame"),
        }
    }
}

/// Collects every plan into an owned DataFrame set.
pub fn lazy_collect_all(
    set: &TerlanPolarsLazyFrameSet,
    streaming: bool,
) -> Result<TerlanPolarsFrameSet, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (set, streaming);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let frames = set
            .plans
            .iter()
            .map(|plan| {
                if streaming {
                    crate::collect_lazy_streaming(plan)
                } else {
                    crate::collect_lazy_in_memory(plan)
                }
            })
            .collect::<Result<Vec<_>, TerlanPolarsError>>()?;
        Ok(TerlanPolarsFrameSet { frames })
    }
}

fn streaming_engine(engine: &str) -> Result<bool, TerlanPolarsError> {
    match engine {
        "auto" | "streaming" => Ok(true),
        "in_memory" | "memory" => Ok(false),
        "gpu" => Err(TerlanPolarsError::new(
            "unsupported_engine",
            "the pinned terlan-polars feature profile does not include the GPU engine",
        )),
        _ => Err(TerlanPolarsError::new(
            "invalid_engine",
            "collection engine must be auto, in_memory, or streaming",
        )),
    }
}

/// Collects one plan with an explicit supported Polars execution engine.
pub fn lazy_collect_with_engine(
    plan: &TerlanPolarsLazyFrame,
    engine: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if engine == "auto" {
        return crate::collect_lazy(plan);
    }
    if streaming_engine(engine)? {
        crate::collect_lazy_streaming(plan)
    } else {
        crate::collect_lazy_in_memory(plan)
    }
}

/// Collects every retained plan with an explicit supported execution engine.
pub fn lazy_collect_all_with_engine(
    set: &TerlanPolarsLazyFrameSet,
    engine: &str,
) -> Result<TerlanPolarsFrameSet, TerlanPolarsError> {
    if engine == "auto" {
        #[cfg(not(feature = "real-polars"))]
        return Err(unavailable());
        #[cfg(feature = "real-polars")]
        {
            let frames = set
                .plans
                .iter()
                .map(crate::collect_lazy)
                .collect::<Result<Vec<_>, TerlanPolarsError>>()?;
            return Ok(TerlanPolarsFrameSet { frames });
        }
    }
    lazy_collect_all(set, streaming_engine(engine)?)
}
