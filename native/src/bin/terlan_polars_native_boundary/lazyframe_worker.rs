//! Native worker dispatch for extended LazyFrame operations.

use super::*;

impl Worker {
    pub(super) fn lazyframe_set_new(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazyframe_set_new expects one LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(plan) => plan,
            Err(error) => return error,
        };
        self.store_result_lazy_frame_set(terlan_polars_native::lazyframe_set_new(plan))
    }

    pub(super) fn lazyframe_set_append(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(set), NativeArg::Handle(plan)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazyframe_set_append expects LazyFrameSet and LazyFrame handles",
            );
        };
        let set = match self.lazy_frame_set(set) {
            Ok(set) => set,
            Err(error) => return error,
        };
        let plan = match self.lazy_frame(plan) {
            Ok(plan) => plan,
            Err(error) => return error,
        };
        self.store_result_lazy_frame_set(terlan_polars_native::lazyframe_set_append(set, plan))
    }

    pub(super) fn lazyframe_set_len(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazyframe_set_len expects one LazyFrameSet handle",
            );
        };
        match self.lazy_frame_set(handle) {
            Ok(set) => format!("ok_int {}", terlan_polars_native::lazyframe_set_len(set)),
            Err(error) => error,
        }
    }

    pub(super) fn lazy_with_context(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(plan), NativeArg::Handle(contexts)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_with_context expects LazyFrame and LazyFrameSet handles",
            );
        };
        let plan = match self.lazy_frame(plan) {
            Ok(plan) => plan,
            Err(error) => return error,
        };
        let contexts = match self.lazy_frame_set(contexts) {
            Ok(contexts) => contexts,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(terlan_polars_native::lazy_with_context(plan, contexts))
    }

    pub(super) fn lazy_collect_all(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(set), NativeArg::Bool(streaming)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_collect_all expects a LazyFrameSet handle and streaming flag",
            );
        };
        let set = match self.lazy_frame_set(set) {
            Ok(set) => set,
            Err(error) => return error,
        };
        match terlan_polars_native::lazy_collect_all(set, *streaming) {
            Ok(frames) => self.store_result_dataframe_set(frames),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn lazy_collect_all_with_engine(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(set), NativeArg::Text(engine)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_collect_all_with_engine expects a LazyFrameSet handle and engine name",
            );
        };
        let set = match self.lazy_frame_set(set) {
            Ok(set) => set,
            Err(error) => return error,
        };
        match terlan_polars_native::lazy_collect_all_with_engine(set, engine) {
            Ok(frames) => self.store_result_dataframe_set(frames),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn lazy_cast(&mut self, args: Vec<NativeArg>, all: bool) -> String {
        let result = match args.as_slice() {
            [NativeArg::Handle(handle), NativeArg::Strings(names), NativeArg::Strings(data_types), NativeArg::Bool(strict)]
                if !all =>
            {
                let plan = match self.lazy_frame(handle) {
                    Ok(plan) => plan,
                    Err(error) => return error,
                };
                lazy_cast_columns(plan, names, data_types, *strict)
            }
            [NativeArg::Handle(handle), NativeArg::Text(data_type), NativeArg::Bool(strict)]
                if all =>
            {
                let plan = match self.lazy_frame(handle) {
                    Ok(plan) => plan,
                    Err(error) => return error,
                };
                lazy_cast_all(plan, data_type, *strict)
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "LazyFrame cast expects a plan, data types, and strict flag",
                );
            }
        };
        match result {
            Ok(plan) => self.store_result_lazy_frame_result(plan),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn lazy_current_optimizations(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_current_optimizations expects one LazyFrame handle",
            );
        };
        match self.lazy_frame(handle) {
            Ok(plan) => {
                let values = terlan_polars_native::lazy_current_optimizations(plan);
                if values.is_empty() {
                    "ok_strings".to_string()
                } else {
                    format!("ok_strings {}", encode_string_list(&values))
                }
            }
            Err(error) => error,
        }
    }

    pub(super) fn lazy_pivot(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(plan), NativeArg::Handle(on_columns), NativeArg::Strings(on), NativeArg::Strings(index), NativeArg::Strings(values), NativeArg::Text(aggregation), NativeArg::Bool(maintain_order), NativeArg::Text(separator), NativeArg::Text(naming)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_pivot expects plan, output columns, selectors, aggregation, ordering, separator, and naming",
            );
        };
        let plan = match self.lazy_frame(plan) {
            Ok(plan) => plan,
            Err(error) => return error,
        };
        let on_columns = match self.frame(on_columns) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match terlan_polars_native::lazy_pivot(
            plan,
            on_columns,
            on,
            index,
            values,
            aggregation,
            *maintain_order,
            separator,
            naming,
        ) {
            Ok(plan) => self.store_result_lazy_frame_result(plan),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }
}
