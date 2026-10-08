//! Immutable expression values shared by Terlan and the Polars adapter.

use serde::{Deserialize, Serialize};

use crate::{TerlanPolarsError, TerlanPolarsScalar};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum ExprNode {
    /// Indexed parameter substituted while applying a declarative expression UDF.
    Parameter {
        index: usize,
    },
    Column {
        name: String,
    },
    Columns {
        names: Vec<String>,
    },
    DTypeColumns {
        data_types: Vec<String>,
    },
    All,
    SelectorLeaf {
        kind: SelectorLeafKind,
        value: String,
    },
    SelectorBinary {
        kind: SelectorSetOp,
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },
    SelectorComplement {
        input: Box<ExprNode>,
    },
    ConcatList {
        inputs: Vec<ExprNode>,
    },
    IntRange {
        start: Box<ExprNode>,
        end: Box<ExprNode>,
        step: i64,
        data_type: String,
    },
    IntRanges {
        start: Box<ExprNode>,
        end: Box<ExprNode>,
        step: Box<ExprNode>,
        data_type: String,
    },
    Literal {
        value: Literal,
    },
    SeriesLiteral {
        ipc_base64: String,
    },
    Date {
        iso: String,
    },
    Sql {
        query: String,
    },
    Len,
    Element,
    Unary {
        kind: UnaryOp,
        input: Box<ExprNode>,
    },
    Binary {
        kind: BinaryOp,
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },
    Alias {
        input: Box<ExprNode>,
        name: String,
    },
    Rename {
        input: Box<ExprNode>,
        value: String,
        prefix: bool,
    },
    KeepName {
        input: Box<ExprNode>,
    },
    ReplaceName {
        input: Box<ExprNode>,
        pattern: String,
        value: String,
        literal: bool,
    },
    LowercaseNames {
        input: Box<ExprNode>,
    },
    MapNames {
        input: Box<ExprNode>,
        fields: bool,
        transform: NameTransformKind,
    },
    StructFieldAffix {
        input: Box<ExprNode>,
        value: String,
        prefix: bool,
    },
    CategoricalAffix {
        input: Box<ExprNode>,
        value: String,
        prefix: bool,
    },
    CategoricalSlice {
        input: Box<ExprNode>,
        offset: i64,
        length: Option<usize>,
    },
    BinarySlice {
        input: Box<ExprNode>,
        offset: Box<ExprNode>,
        length: Box<ExprNode>,
    },
    BinaryDecode {
        input: Box<ExprNode>,
        encoding: BinaryEncoding,
        strict: bool,
    },
    BinaryReinterpret {
        input: Box<ExprNode>,
        data_type: String,
        is_little_endian: bool,
    },
    ArrayStatistic {
        input: Box<ExprNode>,
        ddof: u8,
        variance: bool,
    },
    ArrayJoin {
        input: Box<ExprNode>,
        separator: Box<ExprNode>,
        ignore_nulls: bool,
    },
    ArrayWindow {
        input: Box<ExprNode>,
        first: Box<ExprNode>,
        second: Option<Box<ExprNode>>,
        kind: ArrayWindowKind,
        as_array: bool,
    },
    ArrayExplode {
        input: Box<ExprNode>,
        empty_as_null: bool,
        keep_nulls: bool,
    },
    ArrayEval {
        input: Box<ExprNode>,
        evaluation: Box<ExprNode>,
        as_list: bool,
        aggregate: bool,
    },
    ArraySortOptions {
        input: Box<ExprNode>,
        descending: bool,
        nulls_last: bool,
        maintain_order: bool,
    },
    ArrayBinaryOptions {
        input: Box<ExprNode>,
        value: Box<ExprNode>,
        kind: ArrayBinaryOptionsKind,
        option: bool,
    },
    ListStatistic {
        input: Box<ExprNode>,
        ddof: u8,
        variance: bool,
    },
    ListJoin {
        input: Box<ExprNode>,
        separator: Box<ExprNode>,
        ignore_nulls: bool,
    },
    ListWindow {
        input: Box<ExprNode>,
        first: Box<ExprNode>,
        second: Option<Box<ExprNode>>,
        kind: ListWindowKind,
    },
    ListToArray {
        input: Box<ExprNode>,
        width: usize,
    },
    ListEval {
        input: Box<ExprNode>,
        evaluation: Box<ExprNode>,
        aggregate: bool,
    },
    ListSortOptions {
        input: Box<ExprNode>,
        descending: bool,
        nulls_last: bool,
        maintain_order: bool,
    },
    ListBinaryOptions {
        input: Box<ExprNode>,
        value: Box<ExprNode>,
        kind: ListBinaryOptionsKind,
        option: bool,
    },
    ListSample {
        input: Box<ExprNode>,
        amount: Box<ExprNode>,
        fraction: bool,
        with_replacement: bool,
        shuffle: bool,
        seed: Option<u64>,
    },
    ListGather {
        input: Box<ExprNode>,
        index: Box<ExprNode>,
        offset: Option<Box<ExprNode>>,
        null_on_oob: bool,
    },
    ListDiff {
        input: Box<ExprNode>,
        periods: i64,
        drop_nulls: bool,
    },
    ListToStruct {
        input: Box<ExprNode>,
        names: Vec<String>,
    },
    Random {
        input: Box<ExprNode>,
        amount: Option<Box<ExprNode>>,
        fraction: bool,
        with_replacement: bool,
        shuffle: bool,
        seed: Option<u64>,
    },
    Hash {
        input: Box<ExprNode>,
        seed: u64,
    },
    Statistic {
        input: Box<ExprNode>,
        by: Option<Box<ExprNode>>,
        kind: StatisticKind,
        ddof: Option<u8>,
    },
    Item {
        input: Box<ExprNode>,
        allow_empty: bool,
    },
    Implode {
        input: Box<ExprNode>,
        maintain_order: bool,
    },
    Quantile {
        input: Box<ExprNode>,
        probability: Box<ExprNode>,
        method: RollingQuantileMethod,
    },
    Mode {
        input: Box<ExprNode>,
        maintain_order: bool,
    },
    Rank {
        input: Box<ExprNode>,
        method: RankKind,
        descending: bool,
        seed: Option<u64>,
    },
    TopK {
        input: Box<ExprNode>,
        count: Box<ExprNode>,
        bottom: bool,
    },
    TopKBy {
        input: Box<ExprNode>,
        count: Box<ExprNode>,
        by: Vec<ExprNode>,
        descending: Vec<bool>,
        bottom: bool,
    },
    ReplaceValues {
        input: Box<ExprNode>,
        old: Box<ExprNode>,
        new: Box<ExprNode>,
        default: Option<Box<ExprNode>>,
        strict: bool,
    },
    Cut {
        input: Box<ExprNode>,
        values: Vec<f64>,
        labels: Option<Vec<String>>,
        kind: CutKind,
        left_closed: bool,
        allow_duplicates: bool,
        include_breaks: bool,
    },
    Bin {
        input: Box<ExprNode>,
        method: BinMethodKind,
        labels: Option<Vec<String>>,
        include_intervals: bool,
    },
    Reshape {
        input: Box<ExprNode>,
        dimensions: Vec<i64>,
    },
    Ewm {
        input: Box<ExprNode>,
        kind: EwmKind,
        alpha: f64,
        adjust: bool,
        bias: bool,
        min_samples: usize,
        ignore_nulls: bool,
    },
    EwmBy {
        input: Box<ExprNode>,
        by: Box<ExprNode>,
        kind: EwmByKind,
        half_life: String,
    },
    Cumulative {
        input: Box<ExprNode>,
        kind: CumulativeKind,
        reverse: bool,
    },
    CumulativeEval {
        input: Box<ExprNode>,
        evaluation: Box<ExprNode>,
        min_samples: usize,
    },
    Rolling {
        input: Box<ExprNode>,
        kind: RollingKind,
        window_size: usize,
        min_samples: usize,
        center: bool,
        #[serde(default)]
        weights: Option<Vec<f64>>,
    },
    RollingMap {
        input: Box<ExprNode>,
        udf: String,
        window_size: usize,
        min_samples: usize,
        center: bool,
    },
    RollingGroup {
        input: Box<ExprNode>,
        index_column: Box<ExprNode>,
        period: String,
        offset: String,
        closed: ClosedIntervalKind,
    },
    RollingBy {
        input: Box<ExprNode>,
        by: Box<ExprNode>,
        kind: RollingKind,
        window_size: String,
        min_samples: usize,
        closed: ClosedIntervalKind,
    },
    Histogram {
        input: Box<ExprNode>,
        bins: Option<Box<ExprNode>>,
        bin_count: Option<usize>,
        include_category: bool,
        include_breakpoint: bool,
    },
    Timestamp {
        input: Box<ExprNode>,
        unit: TemporalTimeUnit,
    },
    TemporalTransform {
        input: Box<ExprNode>,
        value: Box<ExprNode>,
        kind: TemporalTransformKind,
    },
    TemporalRange {
        start: Box<ExprNode>,
        end: Box<ExprNode>,
        interval: String,
        closed: ClosedIntervalKind,
        kind: TemporalRangeKind,
        time_unit: Option<TemporalTimeUnit>,
        time_zone: String,
        ranges: bool,
    },
    Duration {
        values: Vec<ExprNode>,
        unit: TemporalTimeUnit,
    },
    DatetimeParts {
        values: Vec<ExprNode>,
        unit: TemporalTimeUnit,
        time_zone: String,
    },
    Repeat {
        value: Box<ExprNode>,
        count: Box<ExprNode>,
    },
    TemporalReplace {
        input: Box<ExprNode>,
        values: Vec<ExprNode>,
    },
    ConvertTimeZone {
        input: Box<ExprNode>,
        time_zone: String,
    },
    ReplaceTimeZone {
        input: Box<ExprNode>,
        time_zone: String,
        ambiguous: Box<ExprNode>,
        non_existent: String,
    },
    DurationTotal {
        input: Box<ExprNode>,
        unit: DurationTotalUnit,
        fractional: bool,
    },
    Exclude {
        input: Box<ExprNode>,
        names: Vec<String>,
    },
    Round {
        input: Box<ExprNode>,
        decimals: u32,
        mode: NumericRoundMode,
    },
    RoundSigFigs {
        input: Box<ExprNode>,
        digits: i32,
    },
    Truncate {
        input: Box<ExprNode>,
        decimals: u32,
    },
    Pi,
    Clip {
        input: Box<ExprNode>,
        min: Option<Box<ExprNode>>,
        max: Option<Box<ExprNode>>,
    },
    Entropy {
        input: Box<ExprNode>,
        base: f64,
        normalize: bool,
    },
    Skew {
        input: Box<ExprNode>,
        bias: bool,
    },
    Kurtosis {
        input: Box<ExprNode>,
        fisher: bool,
        bias: bool,
    },
    Between {
        input: Box<ExprNode>,
        lower: Box<ExprNode>,
        upper: Box<ExprNode>,
        closed: ClosedIntervalKind,
    },
    IsIn {
        input: Box<ExprNode>,
        other: Box<ExprNode>,
        nulls_equal: bool,
    },
    IsClose {
        input: Box<ExprNode>,
        other: Box<ExprNode>,
        absolute_tolerance: f64,
        relative_tolerance: f64,
        nans_equal: bool,
    },
    BooleanReduction {
        input: Box<ExprNode>,
        kind: BooleanReductionKind,
        ignore_nulls: bool,
    },
    SplitFirst {
        input: Box<ExprNode>,
        separator: String,
    },
    StringSplit {
        input: Box<ExprNode>,
        separator: String,
    },
    DateFormat {
        input: Box<ExprNode>,
        format: String,
    },
    UppercaseNames {
        input: Box<ExprNode>,
    },
    Cast {
        input: Box<ExprNode>,
        data_type: String,
        strict: bool,
    },
    CastWithOptions {
        input: Box<ExprNode>,
        data_type: String,
        mode: CastMode,
    },
    FillNullStrategy {
        input: Box<ExprNode>,
        strategy: FillNullStrategyKind,
        limit: Option<u32>,
    },
    SetSortedFlag {
        input: Box<ExprNode>,
        descending: Option<bool>,
        nulls_last: Option<bool>,
    },
    IsSorted {
        input: Box<ExprNode>,
        descending: Option<bool>,
        nulls_last: Option<bool>,
    },
    ParseDatetime {
        input: Box<ExprNode>,
    },
    StringExtract {
        input: Box<ExprNode>,
        pattern: String,
        group: usize,
        all: bool,
    },
    StringReplace {
        input: Box<ExprNode>,
        pattern: String,
        replacement: String,
        all: bool,
    },
    StringSlice {
        input: Box<ExprNode>,
        offset: Box<ExprNode>,
        kind: StringSliceKind,
    },
    StringFind {
        input: Box<ExprNode>,
        pattern: Box<ExprNode>,
        literal: bool,
        strict: bool,
    },
    StringCountMatches {
        input: Box<ExprNode>,
        pattern: Box<ExprNode>,
        literal: bool,
    },
    StringPad {
        input: Box<ExprNode>,
        length: Box<ExprNode>,
        kind: StringPadKind,
    },
    StringDecode {
        input: Box<ExprNode>,
        encoding: BinaryEncoding,
        strict: bool,
    },
    StringNormalize {
        input: Box<ExprNode>,
        form: StringNormalizationForm,
    },
    Fold {
        kind: FoldKind,
        initial: Option<Box<ExprNode>>,
        inputs: Vec<ExprNode>,
        separator: Option<String>,
    },
    FilterExpression {
        input: Box<ExprNode>,
        predicate: Box<ExprNode>,
    },
    SliceExpression {
        input: Box<ExprNode>,
        offset: Box<ExprNode>,
        length: Box<ExprNode>,
    },
    Append {
        input: Box<ExprNode>,
        other: Box<ExprNode>,
        upcast: bool,
    },
    ArgSort {
        input: Box<ExprNode>,
        descending: bool,
        nulls_last: bool,
    },
    SearchSorted {
        input: Box<ExprNode>,
        element: Box<ExprNode>,
        side: SearchSide,
        descending: bool,
    },
    Gather {
        input: Box<ExprNode>,
        index: Box<ExprNode>,
        scalar: bool,
        null_on_oob: bool,
    },
    SortOptions {
        input: Box<ExprNode>,
        descending: bool,
        nulls_last: bool,
        #[serde(default)]
        maintain_order: bool,
    },
    ShiftAndFill {
        input: Box<ExprNode>,
        periods: Box<ExprNode>,
        fill: Box<ExprNode>,
    },
    Diff {
        input: Box<ExprNode>,
        periods: Box<ExprNode>,
        behavior: DiffBehavior,
    },
    GatherEvery {
        input: Box<ExprNode>,
        step: usize,
        offset: usize,
    },
    ExtendConstant {
        input: Box<ExprNode>,
        value: Box<ExprNode>,
        count: Box<ExprNode>,
    },
    SortBy {
        input: Box<ExprNode>,
        by: Vec<ExprNode>,
        descending: bool,
    },
    SortByOptions {
        input: Box<ExprNode>,
        by: Vec<ExprNode>,
        descending: Vec<bool>,
        nulls_last: Vec<bool>,
        maintain_order: bool,
    },
    Over {
        input: Box<ExprNode>,
        keys: Vec<ExprNode>,
        mapping: WindowMappingKind,
    },
    OverOrdered {
        input: Box<ExprNode>,
        partition_by: Vec<ExprNode>,
        order_by: Vec<ExprNode>,
        descending: bool,
        nulls_last: bool,
        mapping: WindowMappingKind,
    },
    HeadExpression {
        input: Box<ExprNode>,
        limit: usize,
    },
    TailExpression {
        input: Box<ExprNode>,
        limit: usize,
    },
    Struct {
        inputs: Vec<ExprNode>,
    },
    StructField {
        input: Box<ExprNode>,
        name: String,
    },
    StructFieldAt {
        input: Box<ExprNode>,
        index: i64,
    },
    StructFields {
        input: Box<ExprNode>,
        names: Vec<String>,
    },
    StructRenameFields {
        input: Box<ExprNode>,
        names: Vec<String>,
    },
    StructWithFields {
        input: Box<ExprNode>,
        fields: Vec<ExprNode>,
    },
    Conditional {
        predicate: Box<ExprNode>,
        truthy: Box<ExprNode>,
        falsy: Box<ExprNode>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum Literal {
    Null,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DataTypeDescriptor {
    List {
        element: String,
    },
    Map {
        key: String,
        value: String,
    },
    Array {
        element: String,
        width: usize,
    },
    Decimal {
        precision: usize,
        scale: usize,
    },
    Categorical,
    Enum {
        categories: Vec<String>,
    },
    Datetime {
        time_unit: String,
        time_zone: String,
    },
    Field {
        name: String,
        data_type: String,
    },
    Struct {
        fields: Vec<String>,
    },
    Extension {
        name: String,
        metadata: Option<String>,
        storage: String,
    },
}

fn encode_data_type_descriptor(
    descriptor: &DataTypeDescriptor,
) -> Result<String, TerlanPolarsError> {
    serde_json::to_string(descriptor).map_err(|error| {
        TerlanPolarsError::new(
            "invalid_data_type",
            format!("cannot encode data type descriptor: {error}"),
        )
    })
}

/// Constructs a Polars list data type descriptor.
pub fn data_type_list(element: &str) -> Result<String, TerlanPolarsError> {
    encode_data_type_descriptor(&DataTypeDescriptor::List {
        element: element.into(),
    })
}

/// Constructs a Polars Map data type descriptor.
pub fn data_type_map(key: &str, value: &str) -> Result<String, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::DataType;

        let data_type = DataType::Map(
            Box::new(parse_data_type(key)?),
            Box::new(parse_data_type(value)?),
        );
        data_type.ensure_valid_map_dtype()?;
    }
    encode_data_type_descriptor(&DataTypeDescriptor::Map {
        key: key.into(),
        value: value.into(),
    })
}

/// Constructs a Polars array data type descriptor.
pub fn data_type_array(element: &str, width: i64) -> Result<String, TerlanPolarsError> {
    let width = usize::try_from(width).map_err(|_| {
        TerlanPolarsError::new("invalid_data_type", "array width cannot be negative")
    })?;
    if width == 0 {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "array width must be greater than zero",
        ));
    }
    encode_data_type_descriptor(&DataTypeDescriptor::Array {
        element: element.into(),
        width,
    })
}

/// Constructs a Polars decimal data type descriptor.
pub fn data_type_decimal(precision: i64, scale: i64) -> Result<String, TerlanPolarsError> {
    let precision = usize::try_from(precision).map_err(|_| {
        TerlanPolarsError::new("invalid_data_type", "decimal precision cannot be negative")
    })?;
    let scale = usize::try_from(scale).map_err(|_| {
        TerlanPolarsError::new("invalid_data_type", "decimal scale cannot be negative")
    })?;
    if !(1..=38).contains(&precision) {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "decimal precision must be between 1 and 38",
        ));
    }
    if scale > precision {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "decimal scale cannot exceed precision",
        ));
    }
    encode_data_type_descriptor(&DataTypeDescriptor::Decimal { precision, scale })
}

/// Constructs a Polars categorical data type descriptor.
pub fn data_type_categorical() -> Result<String, TerlanPolarsError> {
    encode_data_type_descriptor(&DataTypeDescriptor::Categorical)
}

/// Constructs a Polars enum data type descriptor.
pub fn data_type_enum(categories: &[String]) -> Result<String, TerlanPolarsError> {
    if categories.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "enum type requires at least one category",
        ));
    }
    if categories.iter().any(String::is_empty) {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "enum categories cannot be empty",
        ));
    }
    for (index, category) in categories.iter().enumerate() {
        if categories[..index].contains(category) {
            return Err(TerlanPolarsError::new(
                "invalid_data_type",
                format!("enum categories must be unique; duplicate `{category}`"),
            ));
        }
    }
    encode_data_type_descriptor(&DataTypeDescriptor::Enum {
        categories: categories.into(),
    })
}

/// Constructs a Polars datetime data type descriptor.
pub fn data_type_datetime(time_unit: &str, time_zone: &str) -> Result<String, TerlanPolarsError> {
    let time_unit = match time_unit {
        "DatetimeMilliseconds" | "datetime_milliseconds" | "milliseconds" => "milliseconds",
        "DatetimeMicroseconds" | "datetime_microseconds" | "microseconds" => "microseconds",
        "DatetimeNanoseconds" | "datetime_nanoseconds" | "nanoseconds" => "nanoseconds",
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_data_type",
                format!("unsupported datetime time unit `{time_unit}`"),
            ));
        }
    };
    if time_zone.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "timezone-aware datetime requires a non-empty time zone",
        ));
    }
    encode_data_type_descriptor(&DataTypeDescriptor::Datetime {
        time_unit: time_unit.into(),
        time_zone: time_zone.into(),
    })
}

/// Constructs a Polars field data type descriptor.
pub fn data_type_field(name: &str, data_type: &str) -> Result<String, TerlanPolarsError> {
    if name.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "struct field name cannot be empty",
        ));
    }
    encode_data_type_descriptor(&DataTypeDescriptor::Field {
        name: name.into(),
        data_type: data_type.into(),
    })
}

/// Constructs a Polars struct data type descriptor.
pub fn data_type_struct(fields: &[String]) -> Result<String, TerlanPolarsError> {
    if fields.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "struct type requires at least one field",
        ));
    }
    encode_data_type_descriptor(&DataTypeDescriptor::Struct {
        fields: fields.into(),
    })
}

/// Constructs an extension data type backed by an explicit storage type.
pub fn data_type_extension(name: &str, storage: &str) -> Result<String, TerlanPolarsError> {
    data_type_extension_with_metadata(name, None, storage)
}

/// Constructs an extension data type with serialized extension metadata.
pub fn data_type_extension_metadata(
    name: &str,
    metadata: &str,
    storage: &str,
) -> Result<String, TerlanPolarsError> {
    data_type_extension_with_metadata(name, Some(metadata), storage)
}

/// Validates and encodes one extension data type descriptor.
fn data_type_extension_with_metadata(
    name: &str,
    metadata: Option<&str>,
    storage: &str,
) -> Result<String, TerlanPolarsError> {
    if name.is_empty() || name.len() > 256 || name.contains('\0') {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "extension type name must contain 1 through 256 non-NUL bytes",
        ));
    }
    if metadata.is_some_and(|value| value.len() > 65_536 || value.contains('\0')) {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "extension metadata must not exceed 65536 bytes or contain NUL",
        ));
    }
    #[cfg(feature = "real-polars")]
    let _ = parse_data_type(storage).map_err(|error| {
        TerlanPolarsError::new(
            "invalid_data_type",
            format!("invalid extension storage type: {}", error.message()),
        )
    })?;
    #[cfg(not(feature = "real-polars"))]
    let _ = parse_scalar_or_descriptor_shape(storage)?;
    encode_data_type_descriptor(&DataTypeDescriptor::Extension {
        name: name.into(),
        metadata: metadata.map(str::to_owned),
        storage: storage.into(),
    })
}

/// Checks that a storage type has a recognizable scalar or descriptor shape.
#[cfg(not(feature = "real-polars"))]
fn parse_scalar_or_descriptor_shape(value: &str) -> Result<(), TerlanPolarsError> {
    if value.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_data_type",
            "extension storage type cannot be empty",
        ));
    }
    if value.starts_with('{') {
        serde_json::from_str::<DataTypeDescriptor>(value).map_err(|error| {
            TerlanPolarsError::new(
                "invalid_data_type",
                format!("invalid extension storage descriptor: {error}"),
            )
        })?;
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum UnaryOp {
    Millennium,
    Century,
    Year,
    IsLeapYear,
    IsoYear,
    Month,
    DaysInMonth,
    Quarter,
    Week,
    Day,
    Weekday,
    OrdinalDay,
    TimeOfDay,
    CalendarDate,
    LocalDatetime,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
    MonthStart,
    MonthEnd,
    Mean,
    Max,
    Not,
    IsNull,
    IsNotNull,
    IsFinite,
    IsInfinite,
    IsNan,
    IsNotNan,
    IsFirstDistinct,
    IsLastDistinct,
    IsUnique,
    IsDuplicated,
    HasNulls,
    NUnique,
    ApproxNUnique,
    ValueCounts,
    UniqueStable,
    UniqueCounts,
    StringLenBytes,
    StringLenChars,
    StringTitlecase,
    StringLowercase,
    StringUppercase,
    StringHexEncode,
    StringBase64Encode,
    StringReverse,
    StringEscapeRegex,
    NullCount,
    DropNulls,
    DropNans,
    FillNullForward,
    FillNullBackward,
    Interpolate,
    AggGroups,
    PeakMin,
    PeakMax,
    Rle,
    RleId,
    Sum,
    Product,
    Count,
    Len,
    FirstNonNull,
    LastNonNull,
    Unique,
    ArgUnique,
    ArgMin,
    ArgMax,
    Rechunk,
    Reverse,
    Floor,
    Ceil,
    Abs,
    Neg,
    Sqrt,
    Cbrt,
    Sign,
    ToPhysical,
    ArgTrue,
    Log1p,
    Exp,
    UpperBound,
    LowerBound,
    First,
    Last,
    SortAscending,
    RankDenseDescending,
    Explode,
    ListLen,
    ListSum,
    ListMean,
    ListMin,
    ListMax,
    ListSort,
    ListMedian,
    ListFirst,
    ListLast,
    ListArgMin,
    ListArgMax,
    ListDropNulls,
    ListToMap,
    BitwiseCountOnes,
    BitwiseCountZeros,
    BitwiseLeadingOnes,
    BitwiseLeadingZeros,
    BitwiseTrailingOnes,
    BitwiseTrailingZeros,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ArrayLen,
    ArraySum,
    ArrayMean,
    ArrayMin,
    ArrayMax,
    ArraySort,
    ArrayToList,
    ArrayMedian,
    ArrayArgMin,
    ArrayArgMax,
    ArrayToStruct,
    MapEntries,
    MapKeys,
    MapValues,
    MapLen,
    CategoricalCategories,
    CategoricalLenBytes,
    CategoricalLenChars,
    BinarySizeBytes,
    BinaryHexEncode,
    BinaryBase64Encode,
    StructJsonEncode,
    Cos,
    Cot,
    Sin,
    Tan,
    ArcCos,
    ArcSin,
    ArcTan,
    Cosh,
    Sinh,
    Tanh,
    ArcCosh,
    ArcSinh,
    ArcTanh,
    Degrees,
    Radians,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StringSliceKind {
    Slice,
    Head,
    Tail,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StringPadKind {
    Start(char),
    End(char),
    ZFill,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StringNormalizationForm {
    Nfc,
    Nfkc,
    Nfd,
    Nfkd,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BinaryEncoding {
    Hex,
    Base64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ArrayWindowKind {
    Slice,
    Head,
    Tail,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ArrayBinaryOptionsKind {
    Get,
    Contains,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ListWindowKind {
    Slice,
    Head,
    Tail,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ListBinaryOptionsKind {
    Get,
    Contains,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FoldKind {
    Sum,
    Product,
    HorizontalSum,
    HorizontalAll,
    HorizontalAny,
    ConcatString,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StatisticKind {
    Std,
    Var,
    Min,
    Median,
    MinBy,
    MaxBy,
    NanMin,
    NanMax,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CumulativeKind {
    Count,
    Sum,
    Product,
    Min,
    Max,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RollingKind {
    Min,
    Max,
    Mean,
    Sum,
    Median,
    Var,
    Std,
    Quantile {
        probability: f64,
        method: RollingQuantileMethod,
    },
    Rank {
        method: RollingRankKind,
        seed: u64,
    },
    Skew {
        bias: bool,
    },
    Kurtosis {
        fisher: bool,
        bias: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RollingQuantileMethod {
    Nearest,
    Lower,
    Higher,
    Midpoint,
    Linear,
    Equiprobable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RollingRankKind {
    Average,
    Min,
    Max,
    Dense,
    Random,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ClosedIntervalKind {
    Both,
    Left,
    Right,
    None,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NumericRoundMode {
    HalfToEven,
    HalfAwayFromZero,
    ToZero,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SearchSide {
    Any,
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DiffBehavior {
    Ignore,
    Drop,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RankKind {
    Average,
    Min,
    Max,
    Dense,
    Ordinal,
    Random,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CutKind {
    Breaks,
    Quantiles,
    Uniform { bins: usize },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum IntervalBinSpec {
    Floats(Vec<f64>),
    Ints(Vec<i64>),
    Strings(Vec<String>),
    Bools(Vec<bool>),
    Series { ipc_base64: String, len: usize },
    Count(usize),
}

impl IntervalBinSpec {
    fn bin_count(&self) -> usize {
        match self {
            Self::Floats(values) => values.len() + 1,
            Self::Ints(values) => values.len() + 1,
            Self::Strings(values) => values.len() + 1,
            Self::Bools(values) => values.len() + 1,
            Self::Series { len, .. } => len + 1,
            Self::Count(count) => *count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FractionBinSpec {
    Fractions(Vec<f64>),
    Count(usize),
}

impl FractionBinSpec {
    fn bin_count(&self) -> usize {
        match self {
            Self::Fractions(values) => values.len() + 1,
            Self::Count(count) => *count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BinMethodKind {
    Intervals {
        spec: IntervalBinSpec,
        right_closed: bool,
    },
    Quantiles {
        spec: FractionBinSpec,
        right_closed: bool,
    },
    Ranks {
        spec: FractionBinSpec,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EwmKind {
    Sum,
    Mean,
    StandardDeviation,
    Variance,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EwmByKind {
    Sum,
    Mean,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WindowMappingKind {
    GroupsToRows,
    Explode,
    Join,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NameTransformKind {
    Prefix(String),
    Suffix(String),
    Replace { pattern: String, value: String },
    Lowercase,
    Uppercase,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CastMode {
    Strict,
    NonStrict,
    Overflowing,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FillNullStrategyKind {
    Forward,
    Backward,
    Min,
    Max,
    Mean,
    Zero,
    One,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BooleanReductionKind {
    Any,
    All,
    IsEmpty,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TemporalTimeUnit {
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TemporalTransformKind {
    Truncate,
    Round,
    OffsetBy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TemporalRangeKind {
    Date,
    Datetime,
    Time,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SelectorLeafKind {
    All,
    Numeric,
    String,
    StartsWith,
    Contains,
    First,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SelectorSetOp {
    Union,
    Intersection,
    Difference,
    ExclusiveOr,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DurationTotalUnit {
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    TrueDivide,
    FloorDivide,
    Modulo,
    Pow,
    Log,
    Dot,
    IndexOf,
    Shift,
    RepeatBy,
    PctChange,
    Equal,
    NotEqual,
    EqualMissing,
    NotEqualMissing,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    Xor,
    LogicalAnd,
    LogicalOr,
    StringStartsWith,
    StringContains,
    StringContainsLiteral,
    StringEndsWith,
    StringStripChars,
    StringStripCharsStart,
    StringStripCharsEnd,
    StringStripPrefix,
    StringStripSuffix,
    FillNull,
    FillNan,
    InterpolateBy,
    ListGet,
    ListContains,
    ListShift,
    ListCountMatches,
    ListSetUnion,
    ListSetDifference,
    ListSetIntersection,
    ListSetSymmetricDifference,
    ArrayGet,
    ArrayContains,
    ArrayCountMatches,
    ArrayShift,
    MapContainsKey,
    MapGet,
    BinaryContains,
    BinaryStartsWith,
    BinaryEndsWith,
    BinaryGet,
    BinaryHead,
    BinaryTail,
    ArcTan2,
}

fn encode(node: &ExprNode) -> Result<String, TerlanPolarsError> {
    serde_json::to_string(node).map_err(|error| {
        TerlanPolarsError::new(
            "invalid_expression",
            format!("cannot encode expression: {error}"),
        )
    })
}

fn decode(encoded: &str) -> Result<ExprNode, TerlanPolarsError> {
    serde_json::from_str(encoded).map_err(|error| {
        TerlanPolarsError::new(
            "invalid_expression",
            format!("cannot decode expression: {error}"),
        )
    })
}

/// Creates one internal expression-UDF parameter node.
pub(crate) fn expression_parameter(index: usize) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Parameter { index })
}

/// Converts a validated encoded expression into a structural JSON value.
pub(crate) fn expression_value(encoded: &str) -> Result<serde_json::Value, TerlanPolarsError> {
    serde_json::to_value(decode(encoded)?).map_err(|error| {
        TerlanPolarsError::new(
            "invalid_expression",
            format!("cannot inspect expression: {error}"),
        )
    })
}

/// Validates and encodes an expression from a structural JSON value.
pub(crate) fn expression_from_value(value: serde_json::Value) -> Result<String, TerlanPolarsError> {
    let node = serde_json::from_value(value).map_err(|error| {
        TerlanPolarsError::new(
            "invalid_expression",
            format!("cannot construct expression: {error}"),
        )
    })?;
    encode(&node)
}

const MAX_EXPRESSION_TREE_RESULTS: usize = 4_096;
const MAX_EXPRESSION_TREE_RESULT_BYTES: usize = 8 * 1024 * 1024;
const MAX_REWRITTEN_EXPRESSION_BYTES: usize = 1024 * 1024;

fn is_expression_value(value: &serde_json::Value) -> bool {
    value
        .as_object()
        .and_then(|fields| fields.get("op"))
        .is_some_and(serde_json::Value::is_string)
}

fn visit_nested_expression_values(
    value: &serde_json::Value,
    visitor: &mut impl FnMut(&serde_json::Value) -> Result<(), TerlanPolarsError>,
) -> Result<(), TerlanPolarsError> {
    if is_expression_value(value) {
        return visitor(value);
    }
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                visit_nested_expression_values(value, visitor)?;
            }
        }
        serde_json::Value::Object(fields) => {
            for value in fields.values() {
                visit_nested_expression_values(value, visitor)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn visit_expression_children(
    value: &serde_json::Value,
    visitor: &mut impl FnMut(&serde_json::Value) -> Result<(), TerlanPolarsError>,
) -> Result<(), TerlanPolarsError> {
    let fields = value.as_object().ok_or_else(|| {
        TerlanPolarsError::new("invalid_expression", "expression node must be an object")
    })?;
    for value in fields.values() {
        visit_nested_expression_values(value, visitor)?;
    }
    Ok(())
}

fn transform_nested_expression_values(
    value: &mut serde_json::Value,
    transform: &mut impl FnMut(&mut serde_json::Value) -> Result<(), TerlanPolarsError>,
) -> Result<(), TerlanPolarsError> {
    if is_expression_value(value) {
        return transform(value);
    }
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                transform_nested_expression_values(value, transform)?;
            }
        }
        serde_json::Value::Object(fields) => {
            for value in fields.values_mut() {
                transform_nested_expression_values(value, transform)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn transform_expression_children(
    value: &mut serde_json::Value,
    transform: &mut impl FnMut(&mut serde_json::Value) -> Result<(), TerlanPolarsError>,
) -> Result<(), TerlanPolarsError> {
    let fields = value.as_object_mut().ok_or_else(|| {
        TerlanPolarsError::new("invalid_expression", "expression node must be an object")
    })?;
    for value in fields.values_mut() {
        transform_nested_expression_values(value, transform)?;
    }
    Ok(())
}

fn push_expression_result(
    value: &serde_json::Value,
    results: &mut Vec<String>,
    total_bytes: &mut usize,
) -> Result<(), TerlanPolarsError> {
    if results.len() >= MAX_EXPRESSION_TREE_RESULTS {
        return Err(TerlanPolarsError::new(
            "expression_tree_limit",
            format!("expression tree contains more than {MAX_EXPRESSION_TREE_RESULTS} nodes"),
        ));
    }
    let encoded = expression_from_value(value.clone())?;
    *total_bytes = total_bytes.checked_add(encoded.len()).ok_or_else(|| {
        TerlanPolarsError::new(
            "expression_tree_limit",
            "expression tree output is too large",
        )
    })?;
    if *total_bytes > MAX_EXPRESSION_TREE_RESULT_BYTES {
        return Err(TerlanPolarsError::new(
            "expression_tree_limit",
            format!("expression tree output exceeds {MAX_EXPRESSION_TREE_RESULT_BYTES} bytes"),
        ));
    }
    results.push(encoded);
    Ok(())
}

fn collect_expression_nodes(
    value: &serde_json::Value,
    results: &mut Vec<String>,
    total_bytes: &mut usize,
) -> Result<(), TerlanPolarsError> {
    push_expression_result(value, results, total_bytes)?;
    visit_expression_children(value, &mut |child| {
        collect_expression_nodes(child, results, total_bytes)
    })
}

fn apply_tree_udf(value: &mut serde_json::Value, udf: &str) -> Result<(), TerlanPolarsError> {
    let argument = expression_from_value(value.clone())?;
    let rewritten = crate::apply_expression_udf(udf, &[argument])?;
    if rewritten.len() > MAX_REWRITTEN_EXPRESSION_BYTES {
        return Err(TerlanPolarsError::new(
            "expression_tree_limit",
            format!("rewritten expression exceeds {MAX_REWRITTEN_EXPRESSION_BYTES} bytes"),
        ));
    }
    *value = expression_value(&rewritten)?;
    Ok(())
}

fn rewrite_expression_value(
    value: &mut serde_json::Value,
    udf: &str,
) -> Result<(), TerlanPolarsError> {
    transform_expression_children(value, &mut |child| rewrite_expression_value(child, udf))?;
    apply_tree_udf(value, udf)
}

/// Returns every expression node in deterministic root-first traversal order.
pub fn expr_meta_nodes(input: &str) -> Result<Vec<String>, TerlanPolarsError> {
    let value = expression_value(input)?;
    let mut results = Vec::new();
    let mut total_bytes = 0;
    collect_expression_nodes(&value, &mut results, &mut total_bytes)?;
    Ok(results)
}

/// Returns the immediate child expressions of an encoded expression.
pub fn expr_meta_children(input: &str) -> Result<Vec<String>, TerlanPolarsError> {
    let value = expression_value(input)?;
    let mut results = Vec::new();
    let mut total_bytes = 0;
    visit_expression_children(&value, &mut |child| {
        push_expression_result(child, &mut results, &mut total_bytes)
    })?;
    Ok(results)
}

/// Rewrites every expression node bottom-up with a unary declarative UDF.
pub fn expr_rewrite(input: &str, udf: &str) -> Result<String, TerlanPolarsError> {
    let mut value = expression_value(input)?;
    rewrite_expression_value(&mut value, udf)?;
    expression_from_value(value)
}

/// Rewrites only the immediate child expressions with a unary declarative UDF.
pub fn expr_map_children(input: &str, udf: &str) -> Result<String, TerlanPolarsError> {
    let mut value = expression_value(input)?;
    transform_expression_children(&mut value, &mut |child| apply_tree_udf(child, udf))?;
    expression_from_value(value)
}

fn unary(encoded: &str, kind: UnaryOp) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Unary {
        kind,
        input: Box::new(decode(encoded)?),
    })
}

fn binary(left: &str, right: &str, kind: BinaryOp) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Binary {
        kind,
        left: Box::new(decode(left)?),
        right: Box::new(decode(right)?),
    })
}

/// Builds a Polars expression descriptor that applies the col operation.
pub fn expr_col(name: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Column { name: name.into() })
}

/// Builds a Polars expression descriptor that applies the cols operation.
pub fn expr_cols(names: &[String]) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Columns {
        names: names.into(),
    })
}

/// Builds a Polars expression descriptor that applies the dtype cols operation.
pub fn expr_dtype_cols(data_types: &[String]) -> Result<String, TerlanPolarsError> {
    if data_types.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "dtype_cols requires at least one data type",
        ));
    }
    encode(&ExprNode::DTypeColumns {
        data_types: data_types.into(),
    })
}

/// Builds a Polars expression descriptor that applies the all operation.
pub fn expr_all() -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::All)
}

fn selector_leaf(kind: SelectorLeafKind, value: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::SelectorLeaf {
        kind,
        value: value.into(),
    })
}

fn is_selector_node(node: &ExprNode) -> bool {
    matches!(
        node,
        ExprNode::All
            | ExprNode::Columns { .. }
            | ExprNode::DTypeColumns { .. }
            | ExprNode::SelectorLeaf { .. }
            | ExprNode::SelectorBinary { .. }
            | ExprNode::SelectorComplement { .. }
    )
}

/// Converts a column or selector expression into a validated selector descriptor.
pub fn expr_into_selector(input: &str) -> Result<String, TerlanPolarsError> {
    match decode(input)? {
        ExprNode::Column { name } => encode(&ExprNode::Columns { names: vec![name] }),
        node if is_selector_node(&node) => encode(&node),
        _ => Err(TerlanPolarsError::new(
            "invalid_expression",
            "only a column or selector expression can become a selector",
        )),
    }
}

/// Selects all columns as a first-class selector expression.
pub fn expr_selector_all() -> Result<String, TerlanPolarsError> {
    selector_leaf(SelectorLeafKind::All, "")
}

/// Selects columns by exact names for selector set algebra.
pub fn expr_selector_names(names: &[String]) -> Result<String, TerlanPolarsError> {
    if names.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "selector_names requires at least one name",
        ));
    }
    encode(&ExprNode::Columns {
        names: names.into(),
    })
}

/// Selects every numeric column.
pub fn expr_selector_numeric() -> Result<String, TerlanPolarsError> {
    selector_leaf(SelectorLeafKind::Numeric, "")
}

/// Selects every String column.
pub fn expr_selector_string() -> Result<String, TerlanPolarsError> {
    selector_leaf(SelectorLeafKind::String, "")
}

/// Selects columns whose names begin with a literal prefix.
pub fn expr_selector_starts_with(prefix: &str) -> Result<String, TerlanPolarsError> {
    selector_leaf(SelectorLeafKind::StartsWith, prefix)
}

/// Selects columns whose names contain literal text.
pub fn expr_selector_contains(value: &str) -> Result<String, TerlanPolarsError> {
    selector_leaf(SelectorLeafKind::Contains, value)
}

/// Selects the first column.
pub fn expr_selector_first() -> Result<String, TerlanPolarsError> {
    selector_leaf(SelectorLeafKind::First, "")
}

fn selector_binary(
    left: &str,
    right: &str,
    kind: SelectorSetOp,
) -> Result<String, TerlanPolarsError> {
    let left = decode(left)?;
    let right = decode(right)?;
    if !is_selector_node(&left) || !is_selector_node(&right) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "selector set operations require selector expressions",
        ));
    }
    encode(&ExprNode::SelectorBinary {
        kind,
        left: Box::new(left),
        right: Box::new(right),
    })
}

/// Computes the union of two selectors.
pub fn expr_selector_union(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    selector_binary(left, right, SelectorSetOp::Union)
}

/// Computes the intersection of two selectors.
pub fn expr_selector_intersection(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    selector_binary(left, right, SelectorSetOp::Intersection)
}

/// Subtracts the right selector from the left selector.
pub fn expr_selector_difference(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    selector_binary(left, right, SelectorSetOp::Difference)
}

/// Computes the symmetric difference of two selectors.
pub fn expr_selector_exclusive_or(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    selector_binary(left, right, SelectorSetOp::ExclusiveOr)
}

/// Selects every column excluded by the supplied selector.
pub fn expr_selector_complement(input: &str) -> Result<String, TerlanPolarsError> {
    let input = decode(input)?;
    if !is_selector_node(&input) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "selector complement requires a selector expression",
        ));
    }
    encode(&ExprNode::SelectorComplement {
        input: Box::new(input),
    })
}

/// Horizontally combines expressions into a list expression.
pub fn expr_concat_list(inputs: &[String]) -> Result<String, TerlanPolarsError> {
    if inputs.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "concat_list requires at least one expression",
        ));
    }
    encode(&ExprNode::ConcatList {
        inputs: inputs
            .iter()
            .map(|input| decode(input))
            .collect::<Result<_, _>>()?,
    })
}

/// Builds one integer range from scalar start and end expressions.
pub fn expr_int_range(
    start: &str,
    end: &str,
    step: i64,
    data_type: &str,
) -> Result<String, TerlanPolarsError> {
    if step == 0 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "integer range step cannot be zero",
        ));
    }
    if data_type.trim().is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "integer range data type cannot be empty",
        ));
    }
    encode(&ExprNode::IntRange {
        start: Box::new(decode(start)?),
        end: Box::new(decode(end)?),
        step,
        data_type: data_type.into(),
    })
}

/// Builds one integer range per input row.
pub fn expr_int_ranges(
    start: &str,
    end: &str,
    step: &str,
    data_type: &str,
) -> Result<String, TerlanPolarsError> {
    if data_type.trim().is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "integer ranges data type cannot be empty",
        ));
    }
    encode(&ExprNode::IntRanges {
        start: Box::new(decode(start)?),
        end: Box::new(decode(end)?),
        step: Box::new(decode(step)?),
        data_type: data_type.into(),
    })
}

/// Builds a Polars expression descriptor that applies the lit operation.
pub fn expr_lit(value: &TerlanPolarsScalar) -> Result<String, TerlanPolarsError> {
    let value = match value {
        TerlanPolarsScalar::String(value) => Literal::String(value.clone()),
        TerlanPolarsScalar::Int(value) => Literal::Int(*value),
        TerlanPolarsScalar::Float(value) => Literal::Float(*value),
        TerlanPolarsScalar::Bool(value) => Literal::Bool(*value),
    };
    encode(&ExprNode::Literal { value })
}

fn extract_i64_node(node: &ExprNode) -> Result<i64, TerlanPolarsError> {
    match node {
        ExprNode::Literal {
            value: Literal::Int(value),
        } => Ok(*value),
        ExprNode::Binary {
            kind: BinaryOp::Subtract,
            left,
            right,
        } => extract_i64_node(left)?
            .checked_sub(extract_i64_node(right)?)
            .ok_or_else(|| {
                TerlanPolarsError::new(
                    "invalid_expression",
                    "constant integer expression overflows Int64",
                )
            }),
        ExprNode::Cast {
            input, data_type, ..
        } if matches!(
            data_type.as_str(),
            "Int8"
                | "int8"
                | "Int16"
                | "int16"
                | "Int32"
                | "int32"
                | "Int64"
                | "int64"
                | "UInt8"
                | "uint8"
                | "UInt16"
                | "uint16"
                | "UInt32"
                | "uint32"
                | "UInt64"
                | "uint64"
        ) =>
        {
            extract_i64_node(input)
        }
        _ => Err(TerlanPolarsError::new(
            "invalid_expression",
            "expression must be a constant integer literal",
        )),
    }
}

/// Extracts one signed integer from a constant literal expression.
pub fn expr_extract_i64(input: &str) -> Result<i64, TerlanPolarsError> {
    extract_i64_node(&decode(input)?)
}

/// Extracts one non-negative platform-sized integer as a Terlan Int.
pub fn expr_extract_usize(input: &str) -> Result<i64, TerlanPolarsError> {
    let value = expr_extract_i64(input)?;
    usize::try_from(value).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "constant integer expression must be non-negative",
        )
    })?;
    Ok(value)
}

#[cfg(not(feature = "real-polars"))]
pub fn expr_series(_series: &crate::TerlanPolarsSeries) -> Result<String, TerlanPolarsError> {
    Err(crate::unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Builds a Polars expression descriptor that applies the series operation.
pub fn expr_series(series: &crate::TerlanPolarsSeries) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::SeriesLiteral {
        ipc_base64: series_ipc_payload(series)?,
    })
}

#[cfg(feature = "real-polars")]
fn series_ipc_payload(series: &crate::TerlanPolarsSeries) -> Result<String, TerlanPolarsError> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use polars::prelude::{IpcWriter, SerWriter};

    let mut frame = series.inner.clone().into_frame();
    let mut bytes = Vec::new();
    IpcWriter::new(&mut bytes).finish(&mut frame)?;
    Ok(STANDARD.encode(bytes))
}

/// Builds a Polars expression descriptor that applies the null operation.
pub fn expr_null() -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Literal {
        value: Literal::Null,
    })
}

/// Builds a Polars expression descriptor that applies the date operation.
pub fn expr_date(iso: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Date { iso: iso.into() })
}

/// Builds a Date expression from year, month, and day expressions.
pub fn expr_date_components(
    year: &str,
    month: &str,
    day: &str,
) -> Result<String, TerlanPolarsError> {
    let datetime = ExprNode::DatetimeParts {
        values: vec![
            decode(year)?,
            decode(month)?,
            decode(day)?,
            ExprNode::Literal {
                value: Literal::Int(0),
            },
            ExprNode::Literal {
                value: Literal::Int(0),
            },
            ExprNode::Literal {
                value: Literal::Int(0),
            },
            ExprNode::Literal {
                value: Literal::Int(0),
            },
            ExprNode::Literal {
                value: Literal::String("raise".into()),
            },
        ],
        unit: TemporalTimeUnit::Microseconds,
        time_zone: String::new(),
    };
    encode(&ExprNode::Cast {
        input: Box::new(datetime),
        data_type: "Date".into(),
        strict: true,
    })
}

/// Parses one standalone Polars SQL expression lazily.
pub fn expr_sql(query: &str) -> Result<String, TerlanPolarsError> {
    if query.trim().is_empty() || query.len() > 1_048_576 || query.contains('\0') {
        return Err(TerlanPolarsError::new(
            "invalid_sql_query",
            "SQL expression must be non-empty, NUL-free, and at most 1048576 bytes",
        ));
    }
    encode(&ExprNode::Sql {
        query: query.to_string(),
    })
}

/// Builds a Polars expression descriptor that applies the len operation.
pub fn expr_len() -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Len)
}

/// Builds a Polars expression descriptor that applies the element operation.
pub fn expr_element() -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Element)
}

/// Builds a Polars expression descriptor that applies the millennium operation.
pub fn expr_millennium(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Millennium)
}

/// Builds a Polars expression descriptor that applies the century operation.
pub fn expr_century(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Century)
}

/// Builds a Polars expression descriptor that applies the year operation.
pub fn expr_year(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Year)
}

/// Builds a Polars expression descriptor that applies the is leap year operation.
pub fn expr_is_leap_year(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsLeapYear)
}

/// Builds a Polars expression descriptor that applies the iso year operation.
pub fn expr_iso_year(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsoYear)
}

/// Builds a Polars expression descriptor that applies the month operation.
pub fn expr_month(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Month)
}

/// Builds a Polars expression descriptor that applies the days in month operation.
pub fn expr_days_in_month(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::DaysInMonth)
}

/// Builds a Polars expression descriptor that applies the quarter operation.
pub fn expr_quarter(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Quarter)
}

/// Builds a Polars expression descriptor that applies the week operation.
pub fn expr_week(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Week)
}

/// Builds a Polars expression descriptor that applies the day operation.
pub fn expr_day(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Day)
}

/// Builds a Polars expression descriptor that applies the weekday operation.
pub fn expr_weekday(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Weekday)
}

/// Builds a Polars expression descriptor that applies the ordinal day operation.
pub fn expr_ordinal_day(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::OrdinalDay)
}

/// Builds a Polars expression descriptor that applies the time of day operation.
pub fn expr_time_of_day(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::TimeOfDay)
}

/// Builds a Polars expression descriptor that applies the calendar date operation.
pub fn expr_calendar_date(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::CalendarDate)
}

/// Builds a Polars expression descriptor that applies the local datetime operation.
pub fn expr_local_datetime(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::LocalDatetime)
}

/// Builds a Polars expression descriptor that applies the hour operation.
pub fn expr_hour(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Hour)
}

/// Builds a Polars expression descriptor that applies the minute operation.
pub fn expr_minute(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Minute)
}

/// Builds a Polars expression descriptor that applies the second operation.
pub fn expr_second(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Second)
}

/// Builds a Polars expression descriptor that applies the millisecond operation.
pub fn expr_millisecond(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Millisecond)
}

/// Builds a Polars expression descriptor that applies the microsecond operation.
pub fn expr_microsecond(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Microsecond)
}

/// Builds a Polars expression descriptor that applies the nanosecond operation.
pub fn expr_nanosecond(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Nanosecond)
}

/// Builds a Polars expression descriptor that applies the month start operation.
pub fn expr_month_start(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::MonthStart)
}

/// Builds a Polars expression descriptor that applies the month end operation.
pub fn expr_month_end(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::MonthEnd)
}

fn parse_closed_interval(value: &str) -> Result<ClosedIntervalKind, TerlanPolarsError> {
    match value {
        "both" => Ok(ClosedIntervalKind::Both),
        "left" => Ok(ClosedIntervalKind::Left),
        "right" => Ok(ClosedIntervalKind::Right),
        "none" | "neither" => Ok(ClosedIntervalKind::None),
        _ => Err(TerlanPolarsError::new(
            "invalid_expression",
            "closed interval must be both, left, right, or none",
        )),
    }
}

fn parse_temporal_time_unit(value: &str) -> Result<TemporalTimeUnit, TerlanPolarsError> {
    match value {
        "ms" | "milliseconds" => Ok(TemporalTimeUnit::Milliseconds),
        "us" | "microseconds" => Ok(TemporalTimeUnit::Microseconds),
        "ns" | "nanoseconds" => Ok(TemporalTimeUnit::Nanoseconds),
        _ => Err(TerlanPolarsError::new(
            "invalid_expression",
            "time unit must be ms, us, or ns",
        )),
    }
}

fn temporal_range(
    start: &str,
    end: &str,
    interval: &str,
    closed: &str,
    kind: TemporalRangeKind,
    time_unit: Option<TemporalTimeUnit>,
    time_zone: &str,
    ranges: bool,
) -> Result<String, TerlanPolarsError> {
    if interval.trim().is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "temporal range interval cannot be empty",
        ));
    }
    encode(&ExprNode::TemporalRange {
        start: Box::new(decode(start)?),
        end: Box::new(decode(end)?),
        interval: interval.into(),
        closed: parse_closed_interval(closed)?,
        kind,
        time_unit,
        time_zone: time_zone.into(),
        ranges,
    })
}

/// Builds one date range.
pub fn expr_date_range(
    start: &str,
    end: &str,
    interval: &str,
    closed: &str,
) -> Result<String, TerlanPolarsError> {
    temporal_range(
        start,
        end,
        interval,
        closed,
        TemporalRangeKind::Date,
        None,
        "",
        false,
    )
}

/// Builds one date range per input row.
pub fn expr_date_ranges(
    start: &str,
    end: &str,
    interval: &str,
    closed: &str,
) -> Result<String, TerlanPolarsError> {
    temporal_range(
        start,
        end,
        interval,
        closed,
        TemporalRangeKind::Date,
        None,
        "",
        true,
    )
}

/// Builds one datetime range.
pub fn expr_datetime_range(
    start: &str,
    end: &str,
    interval: &str,
    closed: &str,
    time_unit: &str,
    time_zone: &str,
) -> Result<String, TerlanPolarsError> {
    temporal_range(
        start,
        end,
        interval,
        closed,
        TemporalRangeKind::Datetime,
        Some(parse_temporal_time_unit(time_unit)?),
        time_zone,
        false,
    )
}

/// Builds one datetime range per input row.
pub fn expr_datetime_ranges(
    start: &str,
    end: &str,
    interval: &str,
    closed: &str,
    time_unit: &str,
    time_zone: &str,
) -> Result<String, TerlanPolarsError> {
    temporal_range(
        start,
        end,
        interval,
        closed,
        TemporalRangeKind::Datetime,
        Some(parse_temporal_time_unit(time_unit)?),
        time_zone,
        true,
    )
}

/// Builds one time range.
pub fn expr_time_range(
    start: &str,
    end: &str,
    interval: &str,
    closed: &str,
) -> Result<String, TerlanPolarsError> {
    temporal_range(
        start,
        end,
        interval,
        closed,
        TemporalRangeKind::Time,
        None,
        "",
        false,
    )
}

/// Builds one time range per input row.
pub fn expr_time_ranges(
    start: &str,
    end: &str,
    interval: &str,
    closed: &str,
) -> Result<String, TerlanPolarsError> {
    temporal_range(
        start,
        end,
        interval,
        closed,
        TemporalRangeKind::Time,
        None,
        "",
        true,
    )
}

/// Builds a duration expression from week through nanosecond component expressions.
pub fn expr_duration(values: &[String], time_unit: &str) -> Result<String, TerlanPolarsError> {
    if values.len() != 8 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "duration requires exactly eight component expressions",
        ));
    }
    encode(&ExprNode::Duration {
        values: values
            .iter()
            .map(|value| decode(value))
            .collect::<Result<_, _>>()?,
        unit: parse_temporal_time_unit(time_unit)?,
    })
}

/// Builds a Datetime expression from year through microsecond components and
/// an ambiguous-time policy. The output name follows the year expression.
pub fn expr_datetime(
    values: &[String],
    time_unit: &str,
    time_zone: &str,
) -> Result<String, TerlanPolarsError> {
    if values.len() != 8 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "datetime requires exactly eight component expressions",
        ));
    }
    encode(&ExprNode::DatetimeParts {
        values: values
            .iter()
            .map(|value| decode(value))
            .collect::<Result<_, _>>()?,
        unit: parse_temporal_time_unit(time_unit)?,
        time_zone: time_zone.into(),
    })
}

/// Repeats a value expression the number of times specified by another
/// expression. The output name follows the value expression.
pub fn expr_repeat(value: &str, count: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Repeat {
        value: Box::new(decode(value)?),
        count: Box::new(decode(count)?),
    })
}

/// Replaces year through microsecond components and resolves ambiguous local times.
pub fn expr_temporal_replace(input: &str, values: &[String]) -> Result<String, TerlanPolarsError> {
    if values.len() != 8 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "temporal replace requires exactly eight component expressions",
        ));
    }
    encode(&ExprNode::TemporalReplace {
        input: Box::new(decode(input)?),
        values: values
            .iter()
            .map(|value| decode(value))
            .collect::<Result<_, _>>()?,
    })
}

/// Converts a timezone-aware datetime to another timezone.
pub fn expr_convert_time_zone(input: &str, time_zone: &str) -> Result<String, TerlanPolarsError> {
    if time_zone.trim().is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "conversion time zone cannot be empty",
        ));
    }
    encode(&ExprNode::ConvertTimeZone {
        input: Box::new(decode(input)?),
        time_zone: time_zone.into(),
    })
}

/// Assigns or removes a datetime timezone, resolving ambiguous and nonexistent times.
pub fn expr_replace_time_zone(
    input: &str,
    time_zone: &str,
    ambiguous: &str,
    non_existent: &str,
) -> Result<String, TerlanPolarsError> {
    if !matches!(non_existent, "raise" | "null") {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "non-existent time policy must be raise or null",
        ));
    }
    encode(&ExprNode::ReplaceTimeZone {
        input: Box::new(decode(input)?),
        time_zone: time_zone.into(),
        ambiguous: Box::new(decode(ambiguous)?),
        non_existent: non_existent.into(),
    })
}

fn timestamp(input: &str, unit: TemporalTimeUnit) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Timestamp {
        input: Box::new(decode(input)?),
        unit,
    })
}

/// Builds a Polars expression descriptor that applies the timestamp milliseconds operation.
pub fn expr_timestamp_milliseconds(input: &str) -> Result<String, TerlanPolarsError> {
    timestamp(input, TemporalTimeUnit::Milliseconds)
}

/// Builds a Polars expression descriptor that applies the timestamp microseconds operation.
pub fn expr_timestamp_microseconds(input: &str) -> Result<String, TerlanPolarsError> {
    timestamp(input, TemporalTimeUnit::Microseconds)
}

/// Builds a Polars expression descriptor that applies the timestamp nanoseconds operation.
pub fn expr_timestamp_nanoseconds(input: &str) -> Result<String, TerlanPolarsError> {
    timestamp(input, TemporalTimeUnit::Nanoseconds)
}

fn temporal_transform(
    input: &str,
    value: &str,
    kind: TemporalTransformKind,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::TemporalTransform {
        input: Box::new(decode(input)?),
        value: Box::new(decode(value)?),
        kind,
    })
}

/// Builds a Polars expression descriptor that applies the temporal truncate operation.
pub fn expr_temporal_truncate(input: &str, every: &str) -> Result<String, TerlanPolarsError> {
    temporal_transform(input, every, TemporalTransformKind::Truncate)
}

/// Builds a Polars expression descriptor that applies the temporal round operation.
pub fn expr_temporal_round(input: &str, every: &str) -> Result<String, TerlanPolarsError> {
    temporal_transform(input, every, TemporalTransformKind::Round)
}

/// Builds a Polars expression descriptor that applies the temporal offset by operation.
pub fn expr_temporal_offset_by(input: &str, by: &str) -> Result<String, TerlanPolarsError> {
    temporal_transform(input, by, TemporalTransformKind::OffsetBy)
}

fn duration_total(
    input: &str,
    unit: DurationTotalUnit,
    fractional: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::DurationTotal {
        input: Box::new(decode(input)?),
        unit,
        fractional,
    })
}

/// Builds a Polars expression descriptor that applies the total days operation.
pub fn expr_total_days(input: &str, fractional: bool) -> Result<String, TerlanPolarsError> {
    duration_total(input, DurationTotalUnit::Days, fractional)
}

/// Builds a Polars expression descriptor that applies the total hours operation.
pub fn expr_total_hours(input: &str, fractional: bool) -> Result<String, TerlanPolarsError> {
    duration_total(input, DurationTotalUnit::Hours, fractional)
}

/// Builds a Polars expression descriptor that applies the total minutes operation.
pub fn expr_total_minutes(input: &str, fractional: bool) -> Result<String, TerlanPolarsError> {
    duration_total(input, DurationTotalUnit::Minutes, fractional)
}

/// Builds a Polars expression descriptor that applies the total seconds operation.
pub fn expr_total_seconds(input: &str, fractional: bool) -> Result<String, TerlanPolarsError> {
    duration_total(input, DurationTotalUnit::Seconds, fractional)
}

/// Builds a Polars expression descriptor that applies the total milliseconds operation.
pub fn expr_total_milliseconds(input: &str, fractional: bool) -> Result<String, TerlanPolarsError> {
    duration_total(input, DurationTotalUnit::Milliseconds, fractional)
}

/// Builds a Polars expression descriptor that applies the total microseconds operation.
pub fn expr_total_microseconds(input: &str, fractional: bool) -> Result<String, TerlanPolarsError> {
    duration_total(input, DurationTotalUnit::Microseconds, fractional)
}

/// Builds a Polars expression descriptor that applies the total nanoseconds operation.
pub fn expr_total_nanoseconds(input: &str, fractional: bool) -> Result<String, TerlanPolarsError> {
    duration_total(input, DurationTotalUnit::Nanoseconds, fractional)
}

/// Builds a Polars expression descriptor that applies the mean operation.
pub fn expr_mean(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Mean)
}

/// Builds a Polars expression descriptor that applies the max operation.
pub fn expr_max(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Max)
}

/// Builds a Polars expression descriptor that applies the not operation.
pub fn expr_not(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Not)
}

/// Builds a Polars expression descriptor that applies the is null operation.
pub fn expr_is_null(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsNull)
}

/// Builds a Polars expression descriptor that applies the is not null operation.
pub fn expr_is_not_null(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsNotNull)
}

/// Builds a Polars expression descriptor that applies the is finite operation.
pub fn expr_is_finite(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsFinite)
}

/// Builds a Polars expression descriptor that applies the is infinite operation.
pub fn expr_is_infinite(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsInfinite)
}

/// Builds a Polars expression descriptor that applies the is nan operation.
pub fn expr_is_nan(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsNan)
}

/// Builds a Polars expression descriptor that applies the is not nan operation.
pub fn expr_is_not_nan(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsNotNan)
}

/// Builds a Polars expression descriptor that applies the is first distinct operation.
pub fn expr_is_first_distinct(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsFirstDistinct)
}

/// Builds a Polars expression descriptor that applies the is last distinct operation.
pub fn expr_is_last_distinct(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsLastDistinct)
}

/// Builds a Polars expression descriptor that applies the is unique operation.
pub fn expr_is_unique(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsUnique)
}

/// Builds a Polars expression descriptor that applies the is duplicated operation.
pub fn expr_is_duplicated(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::IsDuplicated)
}

/// Builds a Polars expression descriptor that applies the has nulls operation.
pub fn expr_has_nulls(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::HasNulls)
}

fn boolean_reduction(
    input: &str,
    kind: BooleanReductionKind,
    ignore_nulls: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::BooleanReduction {
        input: Box::new(decode(input)?),
        kind,
        ignore_nulls,
    })
}

/// Builds a Polars expression descriptor that applies the any operation.
pub fn expr_any(input: &str, ignore_nulls: bool) -> Result<String, TerlanPolarsError> {
    boolean_reduction(input, BooleanReductionKind::Any, ignore_nulls)
}

/// Builds a Polars expression descriptor that applies the all true operation.
pub fn expr_all_true(input: &str, ignore_nulls: bool) -> Result<String, TerlanPolarsError> {
    boolean_reduction(input, BooleanReductionKind::All, ignore_nulls)
}

/// Builds a Polars expression descriptor that applies the is empty operation.
pub fn expr_is_empty(input: &str, ignore_nulls: bool) -> Result<String, TerlanPolarsError> {
    boolean_reduction(input, BooleanReductionKind::IsEmpty, ignore_nulls)
}

/// Builds a Polars expression descriptor that applies the n unique operation.
pub fn expr_n_unique(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::NUnique)
}

/// Builds a Polars expression descriptor that applies the approx n unique operation.
pub fn expr_approx_n_unique(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ApproxNUnique)
}

/// Builds a Polars expression descriptor that applies the value counts operation.
pub fn expr_value_counts(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ValueCounts)
}

/// Builds a Polars expression descriptor that applies the unique stable operation.
pub fn expr_unique_stable(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::UniqueStable)
}

/// Builds a Polars expression descriptor that applies the unique counts operation.
pub fn expr_unique_counts(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::UniqueCounts)
}

/// Builds a Polars expression descriptor that applies the cos operation.
pub fn expr_cos(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Cos)
}

/// Builds a Polars expression descriptor that applies the cot operation.
pub fn expr_cot(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Cot)
}

/// Builds a Polars expression descriptor that applies the sin operation.
pub fn expr_sin(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Sin)
}

/// Builds a Polars expression descriptor that applies the tan operation.
pub fn expr_tan(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Tan)
}

/// Builds a Polars expression descriptor that applies the arccos operation.
pub fn expr_arccos(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArcCos)
}

/// Builds a Polars expression descriptor that applies the arcsin operation.
pub fn expr_arcsin(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArcSin)
}

/// Builds a Polars expression descriptor that applies the arctan operation.
pub fn expr_arctan(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArcTan)
}

/// Builds a Polars expression descriptor that applies the arctan2 operation.
pub fn expr_arctan2(y: &str, x: &str) -> Result<String, TerlanPolarsError> {
    binary(y, x, BinaryOp::ArcTan2)
}

/// Builds a Polars expression descriptor that applies the cosh operation.
pub fn expr_cosh(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Cosh)
}

/// Builds a Polars expression descriptor that applies the sinh operation.
pub fn expr_sinh(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Sinh)
}

/// Builds a Polars expression descriptor that applies the tanh operation.
pub fn expr_tanh(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Tanh)
}

/// Builds a Polars expression descriptor that applies the arccosh operation.
pub fn expr_arccosh(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArcCosh)
}

/// Builds a Polars expression descriptor that applies the arcsinh operation.
pub fn expr_arcsinh(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArcSinh)
}

/// Builds a Polars expression descriptor that applies the arctanh operation.
pub fn expr_arctanh(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArcTanh)
}

/// Builds a Polars expression descriptor that applies the degrees operation.
pub fn expr_degrees(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Degrees)
}

/// Builds a Polars expression descriptor that applies the radians operation.
pub fn expr_radians(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Radians)
}

/// Builds a Polars expression descriptor that applies the add operation.
pub fn expr_add(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::Add)
}

/// Builds a Polars expression descriptor that applies the subtract operation.
pub fn expr_subtract(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::Subtract)
}

/// Builds a Polars expression descriptor that applies the multiply operation.
pub fn expr_multiply(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::Multiply)
}

/// Builds a Polars expression descriptor that applies the divide operation.
pub fn expr_divide(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::Divide)
}

/// Builds a division expression that always uses true-division semantics.
pub fn expr_true_divide(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::TrueDivide)
}

/// Builds a Polars expression descriptor that applies the floor divide operation.
pub fn expr_floor_divide(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::FloorDivide)
}

/// Builds a Polars expression descriptor that applies the modulo operation.
pub fn expr_modulo(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::Modulo)
}

/// Builds a Polars expression descriptor that applies the pow operation.
pub fn expr_pow(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::Pow)
}

/// Builds a Polars expression descriptor that applies the lt operation.
pub fn expr_lt(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::LessThan)
}

/// Builds a Polars expression descriptor that applies the lte operation.
pub fn expr_lte(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::LessThanOrEqual)
}

/// Builds a Polars expression descriptor that applies the gt operation.
pub fn expr_gt(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::GreaterThan)
}

/// Builds a Polars expression descriptor that applies the gte operation.
pub fn expr_gte(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::GreaterThanOrEqual)
}

/// Builds a Polars expression descriptor that applies the eq operation.
pub fn expr_eq(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::Equal)
}

/// Builds a Polars expression descriptor that applies the neq operation.
pub fn expr_neq(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::NotEqual)
}

/// Builds a Polars expression descriptor that applies the eq missing operation.
pub fn expr_eq_missing(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::EqualMissing)
}

/// Builds a Polars expression descriptor that applies the neq missing operation.
pub fn expr_neq_missing(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::NotEqualMissing)
}

/// Builds a Polars expression descriptor that applies the and operation.
pub fn expr_and(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::And)
}

/// Builds a Polars expression descriptor that applies the or operation.
pub fn expr_or(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::Or)
}

/// Builds a Polars expression descriptor that applies the xor operation.
pub fn expr_xor(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::Xor)
}

/// Builds a Polars expression descriptor that applies the logical and operation.
pub fn expr_logical_and(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::LogicalAnd)
}

/// Builds a Polars expression descriptor that applies the logical or operation.
pub fn expr_logical_or(left: &str, right: &str) -> Result<String, TerlanPolarsError> {
    binary(left, right, BinaryOp::LogicalOr)
}

/// Builds a Polars expression descriptor that applies the when then otherwise operation.
pub fn expr_when_then_otherwise(
    predicate: &str,
    truthy: &str,
    falsy: &str,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Conditional {
        predicate: Box::new(decode(predicate)?),
        truthy: Box::new(decode(truthy)?),
        falsy: Box::new(decode(falsy)?),
    })
}

/// Builds a chained Polars conditional from parallel predicate and value lists.
///
/// The first matching predicate wins. The final expression is used when none
/// of the predicates match, matching Python Polars' chained
/// `when(...).then(...).when(...).then(...).otherwise(...)` semantics.
pub fn expr_when_chain(
    predicates: &[String],
    values: &[String],
    otherwise: &str,
) -> Result<String, TerlanPolarsError> {
    if predicates.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "when_chain requires at least one predicate and value",
        ));
    }
    if predicates.len() != values.len() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "when_chain requires equal predicate and value counts",
        ));
    }

    let mut falsy = decode(otherwise)?;
    for (predicate, truthy) in predicates.iter().zip(values).rev() {
        falsy = ExprNode::Conditional {
            predicate: Box::new(decode(predicate)?),
            truthy: Box::new(decode(truthy)?),
            falsy: Box::new(falsy),
        };
    }
    encode(&falsy)
}

/// Builds a Polars expression descriptor that applies the alias operation.
pub fn expr_alias(input: &str, name: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Alias {
        input: Box::new(decode(input)?),
        name: name.into(),
    })
}

/// Builds a Polars expression descriptor that applies the suffix operation.
pub fn expr_suffix(input: &str, value: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Rename {
        input: Box::new(decode(input)?),
        value: value.into(),
        prefix: false,
    })
}

/// Builds a Polars expression descriptor that applies the prefix operation.
pub fn expr_prefix(input: &str, value: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Rename {
        input: Box::new(decode(input)?),
        value: value.into(),
        prefix: true,
    })
}

/// Builds a Polars expression descriptor that applies the keep name operation.
pub fn expr_keep_name(input: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::KeepName {
        input: Box::new(decode(input)?),
    })
}

/// Builds a Polars expression descriptor that applies the replace name operation.
pub fn expr_replace_name(
    input: &str,
    pattern: &str,
    value: &str,
    literal: bool,
) -> Result<String, TerlanPolarsError> {
    if pattern.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "name replacement pattern cannot be empty",
        ));
    }
    encode(&ExprNode::ReplaceName {
        input: Box::new(decode(input)?),
        pattern: pattern.into(),
        value: value.into(),
        literal,
    })
}

/// Builds a Polars expression descriptor that applies the lowercase names operation.
pub fn expr_lowercase_names(input: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::LowercaseNames {
        input: Box::new(decode(input)?),
    })
}

fn name_transform(
    input: &str,
    fields: bool,
    mode: &str,
    first: &str,
    second: &str,
) -> Result<String, TerlanPolarsError> {
    let transform = match mode {
        "prefix" => NameTransformKind::Prefix(first.into()),
        "suffix" => NameTransformKind::Suffix(first.into()),
        "replace" => {
            if first.is_empty() {
                return Err(TerlanPolarsError::new(
                    "invalid_expression",
                    "name replacement pattern cannot be empty",
                ));
            }
            NameTransformKind::Replace {
                pattern: first.into(),
                value: second.into(),
            }
        }
        "lowercase" => NameTransformKind::Lowercase,
        "uppercase" => NameTransformKind::Uppercase,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "name transform must be prefix, suffix, replace, lowercase, or uppercase",
            ));
        }
    };
    encode(&ExprNode::MapNames {
        input: Box::new(decode(input)?),
        fields,
        transform,
    })
}

/// Applies a serializable transformation to an expression's root name.
pub fn expr_map_name(
    input: &str,
    mode: &str,
    first: &str,
    second: &str,
) -> Result<String, TerlanPolarsError> {
    name_transform(input, false, mode, first, second)
}

/// Applies a serializable transformation to every Struct field name.
pub fn expr_map_field_names(
    input: &str,
    mode: &str,
    first: &str,
    second: &str,
) -> Result<String, TerlanPolarsError> {
    name_transform(input, true, mode, first, second)
}

fn struct_field_affix(input: &str, value: &str, prefix: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StructFieldAffix {
        input: Box::new(decode(input)?),
        value: value.into(),
        prefix,
    })
}

/// Builds a Polars expression descriptor that applies the prefix field names operation.
pub fn expr_prefix_field_names(input: &str, value: &str) -> Result<String, TerlanPolarsError> {
    struct_field_affix(input, value, true)
}

/// Builds a Polars expression descriptor that applies the suffix field names operation.
pub fn expr_suffix_field_names(input: &str, value: &str) -> Result<String, TerlanPolarsError> {
    struct_field_affix(input, value, false)
}

/// Builds a Polars expression descriptor that applies the exclude operation.
pub fn expr_exclude(input: &str, names: &[String]) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Exclude {
        input: Box::new(decode(input)?),
        names: names.into(),
    })
}

/// Builds a Polars expression descriptor that applies the round operation.
pub fn expr_round(input: &str, decimals: i64) -> Result<String, TerlanPolarsError> {
    let decimals = u32::try_from(decimals).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "round decimal count cannot be negative",
        )
    })?;
    encode(&ExprNode::Round {
        input: Box::new(decode(input)?),
        decimals,
        mode: NumericRoundMode::HalfToEven,
    })
}

/// Builds a Polars expression descriptor that applies the round with mode operation.
pub fn expr_round_with_mode(
    input: &str,
    decimals: i64,
    mode: &str,
) -> Result<String, TerlanPolarsError> {
    let decimals = u32::try_from(decimals).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "round decimal count cannot be negative",
        )
    })?;
    let mode = match mode {
        "half_to_even" => NumericRoundMode::HalfToEven,
        "half_away_from_zero" => NumericRoundMode::HalfAwayFromZero,
        "to_zero" => NumericRoundMode::ToZero,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "round mode must be half_to_even, half_away_from_zero, or to_zero",
            ))
        }
    };
    encode(&ExprNode::Round {
        input: Box::new(decode(input)?),
        decimals,
        mode,
    })
}

/// Builds a Polars expression descriptor that applies the round sig figs operation.
pub fn expr_round_sig_figs(input: &str, digits: i64) -> Result<String, TerlanPolarsError> {
    let digits = i32::try_from(digits).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "significant digit count exceeds Polars limits",
        )
    })?;
    if digits < 1 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "significant digit count must be at least one",
        ));
    }
    encode(&ExprNode::RoundSigFigs {
        input: Box::new(decode(input)?),
        digits,
    })
}

/// Builds a Polars expression descriptor that applies the truncate operation.
pub fn expr_truncate(input: &str, decimals: i64) -> Result<String, TerlanPolarsError> {
    let decimals = u32::try_from(decimals).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "truncate decimal count cannot be negative",
        )
    })?;
    encode(&ExprNode::Truncate {
        input: Box::new(decode(input)?),
        decimals,
    })
}

/// Builds a Polars expression descriptor that applies the pi operation.
pub fn expr_pi() -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Pi)
}

/// Builds a Polars expression descriptor that applies the clip operation.
pub fn expr_clip(input: &str, min: &str, max: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Clip {
        input: Box::new(decode(input)?),
        min: Some(Box::new(decode(min)?)),
        max: Some(Box::new(decode(max)?)),
    })
}

/// Builds a Polars expression descriptor that applies the clip min operation.
pub fn expr_clip_min(input: &str, min: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Clip {
        input: Box::new(decode(input)?),
        min: Some(Box::new(decode(min)?)),
        max: None,
    })
}

/// Builds a Polars expression descriptor that applies the clip max operation.
pub fn expr_clip_max(input: &str, max: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Clip {
        input: Box::new(decode(input)?),
        min: None,
        max: Some(Box::new(decode(max)?)),
    })
}

/// Builds a Polars expression descriptor that applies the floor operation.
pub fn expr_floor(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Floor)
}

/// Builds a Polars expression descriptor that applies the ceil operation.
pub fn expr_ceil(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Ceil)
}

/// Builds a Polars expression descriptor that applies the abs operation.
pub fn expr_abs(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Abs)
}

/// Builds an element-wise arithmetic negation expression.
pub fn expr_neg(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Neg)
}

/// Builds an element-wise square-root expression.
pub fn expr_sqrt(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Sqrt)
}

/// Builds an element-wise cube-root expression.
pub fn expr_cbrt(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Cbrt)
}

/// Builds a Polars expression descriptor that applies the sign operation.
pub fn expr_sign(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Sign)
}

/// Converts a logical expression to its physical storage representation.
pub fn expr_to_physical(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ToPhysical)
}

fn parse_sorted_flag(value: &str) -> Result<Option<bool>, TerlanPolarsError> {
    match value {
        "ascending" | "asc" => Ok(Some(false)),
        "descending" | "desc" => Ok(Some(true)),
        "unknown" | "not" | "none" => Ok(None),
        _ => Err(TerlanPolarsError::new(
            "invalid_expression",
            "sorted direction must be ascending, descending, or unknown",
        )),
    }
}

fn parse_null_placement(value: &str) -> Result<Option<bool>, TerlanPolarsError> {
    match value {
        "first" | "nulls_first" => Ok(Some(false)),
        "last" | "nulls_last" => Ok(Some(true)),
        "unknown" | "none" => Ok(None),
        _ => Err(TerlanPolarsError::new(
            "invalid_expression",
            "null placement must be first, last, or unknown",
        )),
    }
}

/// Marks expression output with trusted sortedness metadata without sorting values.
pub fn expr_set_sorted_flag(
    input: &str,
    direction: &str,
    null_placement: &str,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::SetSortedFlag {
        input: Box::new(decode(input)?),
        descending: parse_sorted_flag(direction)?,
        nulls_last: parse_null_placement(null_placement)?,
    })
}

/// Checks whether expression values are sorted in a requested or inferred direction.
pub fn expr_is_sorted(
    input: &str,
    direction: &str,
    null_placement: &str,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::IsSorted {
        input: Box::new(decode(input)?),
        descending: parse_sorted_flag(direction)?,
        nulls_last: parse_null_placement(null_placement)?,
    })
}

/// Builds a Polars expression descriptor that applies the product operation.
pub fn expr_product(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Product)
}

/// Builds a Polars expression descriptor that applies the log operation.
pub fn expr_log(input: &str, base: &str) -> Result<String, TerlanPolarsError> {
    binary(input, base, BinaryOp::Log)
}

/// Builds a base-10 logarithm expression.
pub fn expr_log10(input: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Binary {
        kind: BinaryOp::Log,
        left: Box::new(decode(input)?),
        right: Box::new(ExprNode::Literal {
            value: Literal::Float(10.0),
        }),
    })
}

/// Builds a Polars expression descriptor that applies the log1p operation.
pub fn expr_log1p(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Log1p)
}

/// Builds a Polars expression descriptor that applies the exp operation.
pub fn expr_exp(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Exp)
}

/// Builds a Polars expression descriptor that applies the entropy operation.
pub fn expr_entropy(input: &str, base: f64, normalize: bool) -> Result<String, TerlanPolarsError> {
    if !base.is_finite() || base <= 0.0 || base == 1.0 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "entropy base must be finite, positive, and different from one",
        ));
    }
    encode(&ExprNode::Entropy {
        input: Box::new(decode(input)?),
        base,
        normalize,
    })
}

/// Builds a Polars expression descriptor that applies the skew operation.
pub fn expr_skew(input: &str, bias: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Skew {
        input: Box::new(decode(input)?),
        bias,
    })
}

/// Builds a Polars expression descriptor that applies the kurtosis operation.
pub fn expr_kurtosis(input: &str, fisher: bool, bias: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Kurtosis {
        input: Box::new(decode(input)?),
        fisher,
        bias,
    })
}

/// Builds a Polars expression descriptor that applies the upper bound operation.
pub fn expr_upper_bound(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::UpperBound)
}

/// Builds a Polars expression descriptor that applies the lower bound operation.
pub fn expr_lower_bound(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::LowerBound)
}

/// Builds a Polars expression descriptor that applies the between operation.
pub fn expr_between(input: &str, lower: &str, upper: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Between {
        input: Box::new(decode(input)?),
        lower: Box::new(decode(lower)?),
        upper: Box::new(decode(upper)?),
        closed: ClosedIntervalKind::Both,
    })
}

/// Builds a Polars expression descriptor that applies the between closed operation.
pub fn expr_between_closed(
    input: &str,
    lower: &str,
    upper: &str,
    closed: &str,
) -> Result<String, TerlanPolarsError> {
    let closed = match closed {
        "both" => ClosedIntervalKind::Both,
        "left" => ClosedIntervalKind::Left,
        "right" => ClosedIntervalKind::Right,
        "none" | "neither" => ClosedIntervalKind::None,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "closed interval must be both, left, right, or none",
            ));
        }
    };
    encode(&ExprNode::Between {
        input: Box::new(decode(input)?),
        lower: Box::new(decode(lower)?),
        upper: Box::new(decode(upper)?),
        closed,
    })
}

/// Builds a Polars expression descriptor that applies the is in operation.
pub fn expr_is_in(
    input: &str,
    other: &str,
    nulls_equal: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::IsIn {
        input: Box::new(decode(input)?),
        other: Box::new(decode(other)?),
        nulls_equal,
    })
}

/// Builds a Polars expression descriptor that applies the is close operation.
pub fn expr_is_close(
    input: &str,
    other: &str,
    absolute_tolerance: f64,
    relative_tolerance: f64,
    nans_equal: bool,
) -> Result<String, TerlanPolarsError> {
    if !absolute_tolerance.is_finite()
        || !relative_tolerance.is_finite()
        || absolute_tolerance < 0.0
        || relative_tolerance < 0.0
    {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "is_close tolerances must be finite and non-negative",
        ));
    }
    encode(&ExprNode::IsClose {
        input: Box::new(decode(input)?),
        other: Box::new(decode(other)?),
        absolute_tolerance,
        relative_tolerance,
        nans_equal,
    })
}

/// Builds a Polars expression descriptor that applies the split first operation.
pub fn expr_split_first(input: &str, separator: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::SplitFirst {
        input: Box::new(decode(input)?),
        separator: separator.into(),
    })
}

/// Builds a Polars expression descriptor that applies the string split operation.
pub fn expr_string_split(input: &str, separator: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StringSplit {
        input: Box::new(decode(input)?),
        separator: separator.into(),
    })
}

/// Builds a Polars expression descriptor that applies the date format operation.
pub fn expr_date_format(input: &str, format: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::DateFormat {
        input: Box::new(decode(input)?),
        format: format.into(),
    })
}

/// Builds a Polars expression descriptor that applies the uppercase names operation.
pub fn expr_uppercase_names(input: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::UppercaseNames {
        input: Box::new(decode(input)?),
    })
}

/// Builds a Polars expression descriptor that applies the cast operation.
pub fn expr_cast(input: &str, data_type: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Cast {
        input: Box::new(decode(input)?),
        data_type: data_type.into(),
        strict: false,
    })
}

/// Builds a Polars expression descriptor that applies the strict cast operation.
pub fn expr_strict_cast(input: &str, data_type: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Cast {
        input: Box::new(decode(input)?),
        data_type: data_type.into(),
        strict: true,
    })
}

/// Casts with explicit strict, non-strict, or overflowing behavior.
pub fn expr_cast_with_options(
    input: &str,
    data_type: &str,
    mode: &str,
) -> Result<String, TerlanPolarsError> {
    let mode = match mode {
        "strict" => CastMode::Strict,
        "non_strict" | "nonstrict" => CastMode::NonStrict,
        "overflowing" => CastMode::Overflowing,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "cast mode must be strict, non_strict, or overflowing",
            ));
        }
    };
    encode(&ExprNode::CastWithOptions {
        input: Box::new(decode(input)?),
        data_type: data_type.into(),
        mode,
    })
}

/// Builds a Polars expression descriptor that applies the parse datetime operation.
pub fn expr_parse_datetime(input: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ParseDatetime {
        input: Box::new(decode(input)?),
    })
}

/// Builds a Polars expression descriptor that applies the string len bytes operation.
pub fn expr_string_len_bytes(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::StringLenBytes)
}

/// Builds a Polars expression descriptor that applies the string len chars operation.
pub fn expr_string_len_chars(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::StringLenChars)
}

/// Builds a Polars expression descriptor that applies the string starts with operation.
pub fn expr_string_starts_with(input: &str, prefix: &str) -> Result<String, TerlanPolarsError> {
    binary(input, prefix, BinaryOp::StringStartsWith)
}

/// Builds a Polars expression descriptor that applies the string contains operation.
pub fn expr_string_contains(input: &str, pattern: &str) -> Result<String, TerlanPolarsError> {
    binary(input, pattern, BinaryOp::StringContains)
}

/// Builds a Polars expression descriptor that applies the string contains literal operation.
pub fn expr_string_contains_literal(
    input: &str,
    pattern: &str,
) -> Result<String, TerlanPolarsError> {
    binary(input, pattern, BinaryOp::StringContainsLiteral)
}

/// Builds a Polars expression descriptor that applies the string ends with operation.
pub fn expr_string_ends_with(input: &str, suffix: &str) -> Result<String, TerlanPolarsError> {
    binary(input, suffix, BinaryOp::StringEndsWith)
}

/// Builds a Polars expression descriptor that applies the string extract operation.
pub fn expr_string_extract(
    input: &str,
    pattern: &str,
    group: i64,
) -> Result<String, TerlanPolarsError> {
    let group = usize::try_from(group).map_err(|_| {
        TerlanPolarsError::new("invalid_expression", "extract group cannot be negative")
    })?;
    encode(&ExprNode::StringExtract {
        input: Box::new(decode(input)?),
        pattern: pattern.into(),
        group,
        all: false,
    })
}

/// Builds a Polars expression descriptor that applies the string extract all operation.
pub fn expr_string_extract_all(input: &str, pattern: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StringExtract {
        input: Box::new(decode(input)?),
        pattern: pattern.into(),
        group: 0,
        all: true,
    })
}

fn string_replace(
    input: &str,
    pattern: &str,
    replacement: &str,
    all: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StringReplace {
        input: Box::new(decode(input)?),
        pattern: pattern.into(),
        replacement: replacement.into(),
        all,
    })
}

/// Builds a Polars expression descriptor that applies the string replace operation.
pub fn expr_string_replace(
    input: &str,
    pattern: &str,
    replacement: &str,
) -> Result<String, TerlanPolarsError> {
    string_replace(input, pattern, replacement, false)
}

/// Builds a Polars expression descriptor that applies the string replace all operation.
pub fn expr_string_replace_all(
    input: &str,
    pattern: &str,
    replacement: &str,
) -> Result<String, TerlanPolarsError> {
    string_replace(input, pattern, replacement, true)
}

/// Builds a Polars expression descriptor that applies the string titlecase operation.
pub fn expr_string_titlecase(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::StringTitlecase)
}

/// Builds a Polars expression descriptor that applies the string lowercase operation.
pub fn expr_string_lowercase(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::StringLowercase)
}

/// Builds a Polars expression descriptor that applies the string uppercase operation.
pub fn expr_string_uppercase(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::StringUppercase)
}

/// Builds a Polars expression descriptor that applies the string strip chars operation.
pub fn expr_string_strip_chars(input: &str, chars: &str) -> Result<String, TerlanPolarsError> {
    binary(input, chars, BinaryOp::StringStripChars)
}

/// Builds a Polars expression descriptor that applies the string strip chars start operation.
pub fn expr_string_strip_chars_start(
    input: &str,
    chars: &str,
) -> Result<String, TerlanPolarsError> {
    binary(input, chars, BinaryOp::StringStripCharsStart)
}

/// Builds a Polars expression descriptor that applies the string strip chars end operation.
pub fn expr_string_strip_chars_end(input: &str, chars: &str) -> Result<String, TerlanPolarsError> {
    binary(input, chars, BinaryOp::StringStripCharsEnd)
}

/// Builds a Polars expression descriptor that applies the string strip prefix operation.
pub fn expr_string_strip_prefix(input: &str, prefix: &str) -> Result<String, TerlanPolarsError> {
    binary(input, prefix, BinaryOp::StringStripPrefix)
}

/// Builds a Polars expression descriptor that applies the string strip suffix operation.
pub fn expr_string_strip_suffix(input: &str, suffix: &str) -> Result<String, TerlanPolarsError> {
    binary(input, suffix, BinaryOp::StringStripSuffix)
}

fn string_slice(
    input: &str,
    offset: &str,
    kind: StringSliceKind,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StringSlice {
        input: Box::new(decode(input)?),
        offset: Box::new(decode(offset)?),
        kind,
    })
}

/// Builds a Polars expression descriptor that applies the string slice operation.
pub fn expr_string_slice(input: &str, offset: &str) -> Result<String, TerlanPolarsError> {
    string_slice(input, offset, StringSliceKind::Slice)
}

/// Builds a Polars expression descriptor that applies the string head operation.
pub fn expr_string_head(input: &str, length: &str) -> Result<String, TerlanPolarsError> {
    string_slice(input, length, StringSliceKind::Head)
}

/// Builds a Polars expression descriptor that applies the string tail operation.
pub fn expr_string_tail(input: &str, length: &str) -> Result<String, TerlanPolarsError> {
    string_slice(input, length, StringSliceKind::Tail)
}

fn string_find(
    input: &str,
    pattern: &str,
    literal: bool,
    strict: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StringFind {
        input: Box::new(decode(input)?),
        pattern: Box::new(decode(pattern)?),
        literal,
        strict,
    })
}

/// Builds a Polars expression descriptor that applies the string find literal operation.
pub fn expr_string_find_literal(input: &str, pattern: &str) -> Result<String, TerlanPolarsError> {
    string_find(input, pattern, true, false)
}

/// Builds a Polars expression descriptor that applies the string find operation.
pub fn expr_string_find(
    input: &str,
    pattern: &str,
    strict: bool,
) -> Result<String, TerlanPolarsError> {
    string_find(input, pattern, false, strict)
}

/// Builds a Polars expression descriptor that applies the string count matches operation.
pub fn expr_string_count_matches(
    input: &str,
    pattern: &str,
    literal: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StringCountMatches {
        input: Box::new(decode(input)?),
        pattern: Box::new(decode(pattern)?),
        literal,
    })
}

fn parse_fill_char(fill: &str) -> Result<char, TerlanPolarsError> {
    let mut chars = fill.chars();
    let Some(fill_char) = chars.next() else {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "string padding fill must contain exactly one character",
        ));
    };
    if chars.next().is_some() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "string padding fill must contain exactly one character",
        ));
    }
    Ok(fill_char)
}

fn string_pad(input: &str, length: &str, kind: StringPadKind) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StringPad {
        input: Box::new(decode(input)?),
        length: Box::new(decode(length)?),
        kind,
    })
}

/// Builds a Polars expression descriptor that applies the string pad start operation.
pub fn expr_string_pad_start(
    input: &str,
    length: &str,
    fill: &str,
) -> Result<String, TerlanPolarsError> {
    string_pad(input, length, StringPadKind::Start(parse_fill_char(fill)?))
}

/// Builds a Polars expression descriptor that applies the string pad end operation.
pub fn expr_string_pad_end(
    input: &str,
    length: &str,
    fill: &str,
) -> Result<String, TerlanPolarsError> {
    string_pad(input, length, StringPadKind::End(parse_fill_char(fill)?))
}

/// Builds a Polars expression descriptor that applies the string zfill operation.
pub fn expr_string_zfill(input: &str, length: &str) -> Result<String, TerlanPolarsError> {
    string_pad(input, length, StringPadKind::ZFill)
}

/// Builds a Polars expression descriptor that applies the string hex encode operation.
pub fn expr_string_hex_encode(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::StringHexEncode)
}

/// Builds a Polars expression descriptor that applies the string hex decode operation.
pub fn expr_string_hex_decode(input: &str, strict: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StringDecode {
        input: Box::new(decode(input)?),
        encoding: BinaryEncoding::Hex,
        strict,
    })
}

/// Builds a Polars expression descriptor that applies the string base64 encode operation.
pub fn expr_string_base64_encode(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::StringBase64Encode)
}

/// Builds a Polars expression descriptor that applies the string base64 decode operation.
pub fn expr_string_base64_decode(input: &str, strict: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StringDecode {
        input: Box::new(decode(input)?),
        encoding: BinaryEncoding::Base64,
        strict,
    })
}

/// Builds a Polars expression descriptor that applies the string normalize operation.
pub fn expr_string_normalize(input: &str, form: &str) -> Result<String, TerlanPolarsError> {
    let form = match form.to_ascii_lowercase().as_str() {
        "nfc" => StringNormalizationForm::Nfc,
        "nfkc" => StringNormalizationForm::Nfkc,
        "nfd" => StringNormalizationForm::Nfd,
        "nfkd" => StringNormalizationForm::Nfkd,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                format!("unsupported Unicode normalization form `{form}`"),
            ))
        }
    };
    encode(&ExprNode::StringNormalize {
        input: Box::new(decode(input)?),
        form,
    })
}

/// Builds a Polars expression descriptor that applies the string reverse operation.
pub fn expr_string_reverse(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::StringReverse)
}

/// Builds a Polars expression descriptor that applies the string escape regex operation.
pub fn expr_string_escape_regex(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::StringEscapeRegex)
}

/// Builds a Polars expression descriptor that applies the null count operation.
pub fn expr_null_count(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::NullCount)
}

/// Builds a Polars expression descriptor that applies the drop nulls operation.
pub fn expr_drop_nulls(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::DropNulls)
}

/// Builds a Polars expression descriptor that applies the drop nans operation.
pub fn expr_drop_nans(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::DropNans)
}

/// Builds a Polars expression descriptor that applies the fill null operation.
pub fn expr_fill_null(input: &str, value: &str) -> Result<String, TerlanPolarsError> {
    binary(input, value, BinaryOp::FillNull)
}

/// Builds a Polars expression descriptor that applies the fill null forward operation.
pub fn expr_fill_null_forward(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::FillNullForward)
}

/// Builds a Polars expression descriptor that applies the fill null backward operation.
pub fn expr_fill_null_backward(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::FillNullBackward)
}

/// Fills null values with a named Polars strategy and optional directional limit.
pub fn expr_fill_null_with_strategy(
    input: &str,
    strategy: &str,
    limit: i64,
) -> Result<String, TerlanPolarsError> {
    let limit = if limit < 0 {
        None
    } else {
        Some(u32::try_from(limit).map_err(|_| {
            TerlanPolarsError::new("invalid_expression", "fill-null limit exceeds UInt32")
        })?)
    };
    let strategy = match strategy {
        "forward" => FillNullStrategyKind::Forward,
        "backward" => FillNullStrategyKind::Backward,
        "min" => FillNullStrategyKind::Min,
        "max" => FillNullStrategyKind::Max,
        "mean" => FillNullStrategyKind::Mean,
        "zero" => FillNullStrategyKind::Zero,
        "one" => FillNullStrategyKind::One,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "fill-null strategy must be forward, backward, min, max, mean, zero, or one",
            ));
        }
    };
    if limit.is_some()
        && !matches!(
            strategy,
            FillNullStrategyKind::Forward | FillNullStrategyKind::Backward
        )
    {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "fill-null limits are only valid for forward or backward strategies",
        ));
    }
    encode(&ExprNode::FillNullStrategy {
        input: Box::new(decode(input)?),
        strategy,
        limit,
    })
}

/// Builds a Polars expression descriptor that applies the interpolate operation.
pub fn expr_interpolate(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Interpolate)
}

/// Builds a Polars expression descriptor that applies the agg groups operation.
pub fn expr_agg_groups(input: &str) -> Result<String, TerlanPolarsError> {
    // Decode first so malformed descriptors continue to report the ordinary
    // descriptor error before the removed-API diagnostic.
    let _ = decode(input)?;
    Err(TerlanPolarsError::new(
        "unsupported_expression",
        "agg_groups was removed in Polars 2.0; add a row index before grouping and aggregate that index column",
    ))
}

/// Builds a Polars expression descriptor that applies the interpolate by operation.
pub fn expr_interpolate_by(input: &str, by: &str) -> Result<String, TerlanPolarsError> {
    binary(input, by, BinaryOp::InterpolateBy)
}

/// Builds a Polars expression descriptor that applies the peak min operation.
pub fn expr_peak_min(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::PeakMin)
}

/// Builds a Polars expression descriptor that applies the peak max operation.
pub fn expr_peak_max(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::PeakMax)
}

/// Builds a Polars expression descriptor that applies the rle operation.
pub fn expr_rle(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Rle)
}

/// Builds a Polars expression descriptor that applies the rle id operation.
pub fn expr_rle_id(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::RleId)
}

/// Builds a Polars expression descriptor that applies the reshape operation.
pub fn expr_reshape(input: &str, dimensions: &[i64]) -> Result<String, TerlanPolarsError> {
    if dimensions.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "reshape requires at least one dimension",
        ));
    }
    if dimensions
        .iter()
        .filter(|dimension| **dimension < 0)
        .count()
        > 1
    {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "reshape permits at most one inferred dimension",
        ));
    }
    if dimensions.iter().any(|dimension| *dimension < -1) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "reshape dimensions must be non-negative or exactly -1",
        ));
    }
    encode(&ExprNode::Reshape {
        input: Box::new(decode(input)?),
        dimensions: dimensions.into(),
    })
}

fn ewm_expression(
    input: &str,
    kind: EwmKind,
    alpha: f64,
    adjust: bool,
    bias: bool,
    min_samples: i64,
    ignore_nulls: bool,
) -> Result<String, TerlanPolarsError> {
    if !alpha.is_finite() || !(0.0 < alpha && alpha <= 1.0) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "exponentially weighted alpha must be finite and in the interval (0, 1]",
        ));
    }
    let min_samples = usize::try_from(min_samples).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "exponentially weighted minimum samples cannot be negative",
        )
    })?;
    encode(&ExprNode::Ewm {
        input: Box::new(decode(input)?),
        kind,
        alpha,
        adjust,
        bias,
        min_samples,
        ignore_nulls,
    })
}

/// Builds a Polars expression descriptor that applies the ewm sum operation.
pub fn expr_ewm_sum(
    input: &str,
    alpha: f64,
    adjust: bool,
    min_samples: i64,
    ignore_nulls: bool,
) -> Result<String, TerlanPolarsError> {
    ewm_expression(
        input,
        EwmKind::Sum,
        alpha,
        adjust,
        false,
        min_samples,
        ignore_nulls,
    )
}

/// Builds a Polars expression descriptor that applies the ewm mean operation.
pub fn expr_ewm_mean(
    input: &str,
    alpha: f64,
    adjust: bool,
    min_samples: i64,
    ignore_nulls: bool,
) -> Result<String, TerlanPolarsError> {
    ewm_expression(
        input,
        EwmKind::Mean,
        alpha,
        adjust,
        false,
        min_samples,
        ignore_nulls,
    )
}

/// Builds a Polars expression descriptor that applies the ewm std operation.
pub fn expr_ewm_std(
    input: &str,
    alpha: f64,
    adjust: bool,
    bias: bool,
    min_samples: i64,
    ignore_nulls: bool,
) -> Result<String, TerlanPolarsError> {
    ewm_expression(
        input,
        EwmKind::StandardDeviation,
        alpha,
        adjust,
        bias,
        min_samples,
        ignore_nulls,
    )
}

/// Builds a Polars expression descriptor that applies the ewm var operation.
pub fn expr_ewm_var(
    input: &str,
    alpha: f64,
    adjust: bool,
    bias: bool,
    min_samples: i64,
    ignore_nulls: bool,
) -> Result<String, TerlanPolarsError> {
    ewm_expression(
        input,
        EwmKind::Variance,
        alpha,
        adjust,
        bias,
        min_samples,
        ignore_nulls,
    )
}

/// Builds a Polars expression descriptor that applies the ewm mean by operation.
pub fn expr_ewm_mean_by(
    input: &str,
    by: &str,
    half_life: &str,
) -> Result<String, TerlanPolarsError> {
    if half_life.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "exponentially weighted half-life cannot be empty",
        ));
    }
    encode(&ExprNode::EwmBy {
        input: Box::new(decode(input)?),
        by: Box::new(decode(by)?),
        kind: EwmByKind::Mean,
        half_life: half_life.into(),
    })
}

/// Builds a time-indexed exponentially weighted moving sum expression.
pub fn expr_ewm_sum_by(
    input: &str,
    by: &str,
    half_life: &str,
) -> Result<String, TerlanPolarsError> {
    if half_life.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "exponentially weighted half-life cannot be empty",
        ));
    }
    encode(&ExprNode::EwmBy {
        input: Box::new(decode(input)?),
        by: Box::new(decode(by)?),
        kind: EwmByKind::Sum,
        half_life: half_life.into(),
    })
}

/// Builds a Polars expression descriptor that applies the cumulative eval operation.
pub fn expr_cumulative_eval(
    input: &str,
    evaluation: &str,
    min_samples: i64,
) -> Result<String, TerlanPolarsError> {
    let min_samples = usize::try_from(min_samples).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "cumulative evaluation minimum samples cannot be negative",
        )
    })?;
    encode(&ExprNode::CumulativeEval {
        input: Box::new(decode(input)?),
        evaluation: Box::new(decode(evaluation)?),
        min_samples,
    })
}

/// Builds a Polars expression descriptor that applies the fill nan operation.
pub fn expr_fill_nan(input: &str, value: &str) -> Result<String, TerlanPolarsError> {
    binary(input, value, BinaryOp::FillNan)
}

/// Builds a Polars expression descriptor that applies the sum operation.
pub fn expr_sum(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Sum)
}

/// Builds a Polars expression descriptor that applies the count operation.
pub fn expr_count(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Count)
}

/// Builds a Polars expression descriptor that applies the len values operation.
pub fn expr_len_values(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Len)
}

/// Builds a Polars expression descriptor that applies the first non null operation.
pub fn expr_first_non_null(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::FirstNonNull)
}

/// Builds a Polars expression descriptor that applies the last non null operation.
pub fn expr_last_non_null(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::LastNonNull)
}

/// Builds a Polars expression descriptor that applies the unique operation.
pub fn expr_unique(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Unique)
}

/// Builds a Polars expression descriptor that applies the arg unique operation.
pub fn expr_arg_unique(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArgUnique)
}

/// Returns row indices for which a boolean expression is true.
pub fn expr_arg_true(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArgTrue)
}

/// Builds a Polars expression descriptor that applies the arg min operation.
pub fn expr_arg_min(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArgMin)
}

/// Builds a Polars expression descriptor that applies the arg max operation.
pub fn expr_arg_max(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArgMax)
}

/// Builds a Polars expression descriptor that applies the item operation.
pub fn expr_item(input: &str, allow_empty: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Item {
        input: Box::new(decode(input)?),
        allow_empty,
    })
}

/// Builds a Polars expression descriptor that applies the implode operation.
pub fn expr_implode(input: &str, maintain_order: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Implode {
        input: Box::new(decode(input)?),
        maintain_order,
    })
}

fn parse_quantile_method(method: &str) -> Result<RollingQuantileMethod, TerlanPolarsError> {
    match method {
        "nearest" => Ok(RollingQuantileMethod::Nearest),
        "lower" => Ok(RollingQuantileMethod::Lower),
        "higher" => Ok(RollingQuantileMethod::Higher),
        "midpoint" => Ok(RollingQuantileMethod::Midpoint),
        "linear" => Ok(RollingQuantileMethod::Linear),
        "equiprobable" => Ok(RollingQuantileMethod::Equiprobable),
        _ => Err(TerlanPolarsError::new(
            "invalid_expression",
            "quantile method must be nearest, lower, higher, midpoint, linear, or equiprobable",
        )),
    }
}

/// Builds a Polars expression descriptor that applies the quantile operation.
pub fn expr_quantile(
    input: &str,
    probability: &str,
    method: &str,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Quantile {
        input: Box::new(decode(input)?),
        probability: Box::new(decode(probability)?),
        method: parse_quantile_method(method)?,
    })
}

/// Builds a Polars expression descriptor that applies the mode operation.
pub fn expr_mode(input: &str, maintain_order: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Mode {
        input: Box::new(decode(input)?),
        maintain_order,
    })
}

fn parse_rank_method(method: &str) -> Result<RankKind, TerlanPolarsError> {
    match method {
        "average" => Ok(RankKind::Average),
        "min" => Ok(RankKind::Min),
        "max" => Ok(RankKind::Max),
        "dense" => Ok(RankKind::Dense),
        "ordinal" => Ok(RankKind::Ordinal),
        "random" => Err(TerlanPolarsError::new(
            "invalid_expression",
            "random rank requires rank_random with an explicit seed",
        )),
        _ => Err(TerlanPolarsError::new(
            "invalid_expression",
            "rank method must be average, min, max, dense, or ordinal",
        )),
    }
}

/// Builds a Polars expression descriptor that applies the rank operation.
pub fn expr_rank(input: &str, method: &str, descending: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Rank {
        input: Box::new(decode(input)?),
        method: parse_rank_method(method)?,
        descending,
        seed: None,
    })
}

/// Builds a Polars expression descriptor that applies the rank random operation.
pub fn expr_rank_random(
    input: &str,
    descending: bool,
    seed: i64,
) -> Result<String, TerlanPolarsError> {
    let seed = u64::try_from(seed).map_err(|_| {
        TerlanPolarsError::new("invalid_expression", "rank seed cannot be negative")
    })?;
    encode(&ExprNode::Rank {
        input: Box::new(decode(input)?),
        method: RankKind::Random,
        descending,
        seed: Some(seed),
    })
}

fn top_k(input: &str, count: &str, bottom: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::TopK {
        input: Box::new(decode(input)?),
        count: Box::new(decode(count)?),
        bottom,
    })
}

/// Builds a Polars expression descriptor that applies the top k operation.
pub fn expr_top_k(input: &str, count: &str) -> Result<String, TerlanPolarsError> {
    top_k(input, count, false)
}

/// Builds a Polars expression descriptor that applies the bottom k operation.
pub fn expr_bottom_k(input: &str, count: &str) -> Result<String, TerlanPolarsError> {
    top_k(input, count, true)
}

fn top_k_by(
    input: &str,
    count: &str,
    by: &[String],
    descending: &[bool],
    bottom: bool,
) -> Result<String, TerlanPolarsError> {
    if by.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "top/bottom-k-by requires at least one ordering expression",
        ));
    }
    if by.len() != descending.len() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "top/bottom-k-by requires one direction for every ordering expression",
        ));
    }
    encode(&ExprNode::TopKBy {
        input: Box::new(decode(input)?),
        count: Box::new(decode(count)?),
        by: decode_many(by)?,
        descending: descending.into(),
        bottom,
    })
}

/// Builds a Polars expression descriptor that applies the top k by operation.
pub fn expr_top_k_by(
    input: &str,
    count: &str,
    by: &[String],
    descending: &[bool],
) -> Result<String, TerlanPolarsError> {
    top_k_by(input, count, by, descending, false)
}

/// Builds a Polars expression descriptor that applies the bottom k by operation.
pub fn expr_bottom_k_by(
    input: &str,
    count: &str,
    by: &[String],
    descending: &[bool],
) -> Result<String, TerlanPolarsError> {
    top_k_by(input, count, by, descending, true)
}

/// Builds a Polars expression descriptor that applies the replace operation.
pub fn expr_replace(input: &str, old: &str, new: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ReplaceValues {
        input: Box::new(decode(input)?),
        old: Box::new(decode(old)?),
        new: Box::new(decode(new)?),
        default: None,
        strict: false,
    })
}

/// Builds a Polars expression descriptor that applies the replace strict operation.
pub fn expr_replace_strict(input: &str, old: &str, new: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ReplaceValues {
        input: Box::new(decode(input)?),
        old: Box::new(decode(old)?),
        new: Box::new(decode(new)?),
        default: None,
        strict: true,
    })
}

/// Builds a Polars expression descriptor that applies the replace or default operation.
pub fn expr_replace_or_default(
    input: &str,
    old: &str,
    new: &str,
    default: &str,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ReplaceValues {
        input: Box::new(decode(input)?),
        old: Box::new(decode(old)?),
        new: Box::new(decode(new)?),
        default: Some(Box::new(decode(default)?)),
        strict: true,
    })
}

fn validate_cut_values(values: &[f64], quantiles: bool) -> Result<(), TerlanPolarsError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            if quantiles {
                "qcut probabilities must be finite"
            } else {
                "cut breaks must be finite"
            },
        ));
    }
    if quantiles && values.iter().any(|value| !(0.0..=1.0).contains(value)) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "qcut probabilities must be between zero and one",
        ));
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            if quantiles {
                "qcut probabilities must be unique"
            } else {
                "cut breaks must be unique"
            },
        ));
    }
    Ok(())
}

fn cut_expression(
    input: &str,
    values: &[f64],
    labels: Option<&[String]>,
    kind: CutKind,
    left_closed: bool,
    allow_duplicates: bool,
    include_breaks: bool,
) -> Result<String, TerlanPolarsError> {
    let expected_labels = match &kind {
        CutKind::Breaks => {
            validate_cut_values(values, false)?;
            values.len() + 1
        }
        CutKind::Quantiles => {
            validate_cut_values(values, true)?;
            values.len() + 1
        }
        CutKind::Uniform { bins } => *bins,
    };
    if let Some(labels) = labels {
        if labels.len() != expected_labels {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                format!("bin label count must be exactly {expected_labels}"),
            ));
        }
    }
    encode(&ExprNode::Cut {
        input: Box::new(decode(input)?),
        values: values.into(),
        labels: labels.map(Into::into),
        kind,
        left_closed,
        allow_duplicates,
        include_breaks,
    })
}

/// Builds a Polars expression descriptor that applies the cut operation.
pub fn expr_cut(
    input: &str,
    breaks: &[f64],
    left_closed: bool,
    include_breaks: bool,
) -> Result<String, TerlanPolarsError> {
    cut_expression(
        input,
        breaks,
        None,
        CutKind::Breaks,
        left_closed,
        false,
        include_breaks,
    )
}

/// Builds a Polars expression descriptor that applies the cut labeled operation.
pub fn expr_cut_labeled(
    input: &str,
    breaks: &[f64],
    labels: &[String],
    left_closed: bool,
    include_breaks: bool,
) -> Result<String, TerlanPolarsError> {
    cut_expression(
        input,
        breaks,
        Some(labels),
        CutKind::Breaks,
        left_closed,
        false,
        include_breaks,
    )
}

/// Builds a Polars expression descriptor that applies the qcut operation.
pub fn expr_qcut(
    input: &str,
    probabilities: &[f64],
    left_closed: bool,
    allow_duplicates: bool,
    include_breaks: bool,
) -> Result<String, TerlanPolarsError> {
    cut_expression(
        input,
        probabilities,
        None,
        CutKind::Quantiles,
        left_closed,
        allow_duplicates,
        include_breaks,
    )
}

/// Builds a Polars expression descriptor that applies the qcut labeled operation.
pub fn expr_qcut_labeled(
    input: &str,
    probabilities: &[f64],
    labels: &[String],
    left_closed: bool,
    allow_duplicates: bool,
    include_breaks: bool,
) -> Result<String, TerlanPolarsError> {
    cut_expression(
        input,
        probabilities,
        Some(labels),
        CutKind::Quantiles,
        left_closed,
        allow_duplicates,
        include_breaks,
    )
}

fn qcut_uniform(
    input: &str,
    bins: i64,
    labels: Option<&[String]>,
    left_closed: bool,
    allow_duplicates: bool,
    include_breaks: bool,
) -> Result<String, TerlanPolarsError> {
    let bins = usize::try_from(bins).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "uniform qcut bin count must be positive",
        )
    })?;
    if bins == 0 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "uniform qcut bin count must be positive",
        ));
    }
    cut_expression(
        input,
        &[],
        labels,
        CutKind::Uniform { bins },
        left_closed,
        allow_duplicates,
        include_breaks,
    )
}

/// Builds a Polars expression descriptor that applies the qcut equal frequency operation.
pub fn expr_qcut_equal_frequency(
    input: &str,
    bins: i64,
    left_closed: bool,
    allow_duplicates: bool,
    include_breaks: bool,
) -> Result<String, TerlanPolarsError> {
    qcut_uniform(
        input,
        bins,
        None,
        left_closed,
        allow_duplicates,
        include_breaks,
    )
}

/// Builds a Polars expression descriptor that applies the qcut equal frequency labeled operation.
pub fn expr_qcut_equal_frequency_labeled(
    input: &str,
    bins: i64,
    labels: &[String],
    left_closed: bool,
    allow_duplicates: bool,
    include_breaks: bool,
) -> Result<String, TerlanPolarsError> {
    qcut_uniform(
        input,
        bins,
        Some(labels),
        left_closed,
        allow_duplicates,
        include_breaks,
    )
}

fn positive_bin_count(value: i64) -> Result<usize, TerlanPolarsError> {
    let count = usize::try_from(value)
        .map_err(|_| TerlanPolarsError::new("invalid_expression", "bin count must be positive"))?;
    if count == 0 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "bin count must be positive",
        ));
    }
    Ok(count)
}

fn validate_bin_labels(
    labels: Option<&[String]>,
    expected: usize,
) -> Result<(), TerlanPolarsError> {
    if labels.is_some_and(|labels| labels.len() != expected) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            format!("bin label count must be exactly {expected}"),
        ));
    }
    Ok(())
}

fn validate_fraction_bins(values: &[f64]) -> Result<(), TerlanPolarsError> {
    if values
        .iter()
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "bin fractions must be finite values between zero and one",
        ));
    }
    if values.windows(2).any(|pair| pair[0] > pair[1]) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "bin fractions must be non-decreasing",
        ));
    }
    Ok(())
}

fn validate_float_breaks(values: &[f64]) -> Result<(), TerlanPolarsError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "interval breakpoints must be finite",
        ));
    }
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "interval breakpoints must be strictly ascending",
        ));
    }
    Ok(())
}

fn validate_ordered_breaks<T: PartialOrd>(values: &[T]) -> Result<(), TerlanPolarsError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "interval breakpoints must be strictly ascending",
        ));
    }
    Ok(())
}

fn bin_expression(
    input: &str,
    method: BinMethodKind,
    labels: Option<&[String]>,
    include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    let count = match &method {
        BinMethodKind::Intervals { spec, .. } => spec.bin_count(),
        BinMethodKind::Quantiles { spec, .. } | BinMethodKind::Ranks { spec } => spec.bin_count(),
    };
    validate_bin_labels(labels, count)?;
    encode(&ExprNode::Bin {
        input: Box::new(decode(input)?),
        method,
        labels: labels.map(Into::into),
        include_intervals,
    })
}

fn bin_intervals_expression(
    input: &str,
    spec: IntervalBinSpec,
    labels: Option<&[String]>,
    right_closed: bool,
    include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    bin_expression(
        input,
        BinMethodKind::Intervals { spec, right_closed },
        labels,
        include_intervals,
    )
}

/// Bins values at floating-point breakpoints and returns integer bin indices.
pub fn expr_bin_intervals(
    input: &str,
    breaks: &[f64],
    right_closed: bool,
    include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    validate_float_breaks(breaks)?;
    bin_intervals_expression(
        input,
        IntervalBinSpec::Floats(breaks.into()),
        None,
        right_closed,
        include_intervals,
    )
}

/// Bins values at floating-point breakpoints and returns the supplied labels.
pub fn expr_bin_intervals_labeled(
    input: &str,
    breaks: &[f64],
    labels: &[String],
    right_closed: bool,
    include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    validate_float_breaks(breaks)?;
    bin_intervals_expression(
        input,
        IntervalBinSpec::Floats(breaks.into()),
        Some(labels),
        right_closed,
        include_intervals,
    )
}

#[cfg(not(feature = "real-polars"))]
/// Binning with Series breakpoints requires the real Polars engine.
pub fn expr_bin_intervals_series(
    _input: &str,
    _breaks: &crate::TerlanPolarsSeries,
    _right_closed: bool,
    _include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    Err(crate::unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Bins values at an arbitrary orderable, non-nested breakpoint Series.
pub fn expr_bin_intervals_series(
    input: &str,
    breaks: &crate::TerlanPolarsSeries,
    right_closed: bool,
    include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    bin_intervals_expression(
        input,
        IntervalBinSpec::Series {
            ipc_base64: series_ipc_payload(breaks)?,
            len: breaks.inner.len(),
        },
        None,
        right_closed,
        include_intervals,
    )
}

#[cfg(not(feature = "real-polars"))]
/// Labeled binning with Series breakpoints requires the real Polars engine.
pub fn expr_bin_intervals_series_labeled(
    _input: &str,
    _breaks: &crate::TerlanPolarsSeries,
    _labels: &[String],
    _right_closed: bool,
    _include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    Err(crate::unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Bins values at an arbitrary breakpoint Series and returns caller labels.
pub fn expr_bin_intervals_series_labeled(
    input: &str,
    breaks: &crate::TerlanPolarsSeries,
    labels: &[String],
    right_closed: bool,
    include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    bin_intervals_expression(
        input,
        IntervalBinSpec::Series {
            ipc_base64: series_ipc_payload(breaks)?,
            len: breaks.inner.len(),
        },
        Some(labels),
        right_closed,
        include_intervals,
    )
}

macro_rules! interval_bin_builders {
    ($plain:ident, $labeled:ident, $ty:ty, $variant:ident) => {
        /// Bins values at typed explicit breakpoints and returns integer indices.
        pub fn $plain(
            input: &str,
            breaks: &[$ty],
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<String, TerlanPolarsError> {
            validate_ordered_breaks(breaks)?;
            bin_intervals_expression(
                input,
                IntervalBinSpec::$variant(breaks.into()),
                None,
                right_closed,
                include_intervals,
            )
        }

        /// Bins values at typed explicit breakpoints and returns caller labels.
        pub fn $labeled(
            input: &str,
            breaks: &[$ty],
            labels: &[String],
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<String, TerlanPolarsError> {
            validate_ordered_breaks(breaks)?;
            bin_intervals_expression(
                input,
                IntervalBinSpec::$variant(breaks.into()),
                Some(labels),
                right_closed,
                include_intervals,
            )
        }
    };
}

interval_bin_builders!(
    expr_bin_intervals_int,
    expr_bin_intervals_int_labeled,
    i64,
    Ints
);
interval_bin_builders!(
    expr_bin_intervals_string,
    expr_bin_intervals_string_labeled,
    String,
    Strings
);
interval_bin_builders!(
    expr_bin_intervals_bool,
    expr_bin_intervals_bool_labeled,
    bool,
    Bools
);

/// Bins numeric values into a positive number of equal-width intervals.
pub fn expr_bin_intervals_uniform(
    input: &str,
    bins: i64,
    right_closed: bool,
    include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    bin_intervals_expression(
        input,
        IntervalBinSpec::Count(positive_bin_count(bins)?),
        None,
        right_closed,
        include_intervals,
    )
}

/// Bins numeric values into labeled equal-width intervals.
pub fn expr_bin_intervals_uniform_labeled(
    input: &str,
    bins: i64,
    labels: &[String],
    right_closed: bool,
    include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    bin_intervals_expression(
        input,
        IntervalBinSpec::Count(positive_bin_count(bins)?),
        Some(labels),
        right_closed,
        include_intervals,
    )
}

fn fraction_bin_expression(
    input: &str,
    spec: FractionBinSpec,
    labels: Option<&[String]>,
    right_closed: Option<bool>,
    include_intervals: bool,
) -> Result<String, TerlanPolarsError> {
    let method = match right_closed {
        Some(right_closed) => BinMethodKind::Quantiles { spec, right_closed },
        None => BinMethodKind::Ranks { spec },
    };
    bin_expression(input, method, labels, include_intervals)
}

macro_rules! fraction_bin_builders {
    ($plain:ident, $labeled:ident, $uniform:ident, $uniform_labeled:ident, $right:expr) => {
        /// Bins values at explicit cumulative fractions and returns integer indices.
        pub fn $plain(
            input: &str,
            fractions: &[f64],
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<String, TerlanPolarsError> {
            validate_fraction_bins(fractions)?;
            fraction_bin_expression(
                input,
                FractionBinSpec::Fractions(fractions.into()),
                None,
                $right(right_closed),
                include_intervals,
            )
        }

        /// Bins values at explicit cumulative fractions and returns caller labels.
        pub fn $labeled(
            input: &str,
            fractions: &[f64],
            labels: &[String],
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<String, TerlanPolarsError> {
            validate_fraction_bins(fractions)?;
            fraction_bin_expression(
                input,
                FractionBinSpec::Fractions(fractions.into()),
                Some(labels),
                $right(right_closed),
                include_intervals,
            )
        }

        /// Bins values into a positive number of uniform bins and returns indices.
        pub fn $uniform(
            input: &str,
            bins: i64,
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<String, TerlanPolarsError> {
            fraction_bin_expression(
                input,
                FractionBinSpec::Count(positive_bin_count(bins)?),
                None,
                $right(right_closed),
                include_intervals,
            )
        }

        /// Bins values into a positive number of uniform bins and returns labels.
        pub fn $uniform_labeled(
            input: &str,
            bins: i64,
            labels: &[String],
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<String, TerlanPolarsError> {
            fraction_bin_expression(
                input,
                FractionBinSpec::Count(positive_bin_count(bins)?),
                Some(labels),
                $right(right_closed),
                include_intervals,
            )
        }
    };
}

fraction_bin_builders!(
    expr_bin_quantiles,
    expr_bin_quantiles_labeled,
    expr_bin_quantiles_uniform,
    expr_bin_quantiles_uniform_labeled,
    Some
);
fraction_bin_builders!(
    expr_bin_ranks,
    expr_bin_ranks_labeled,
    expr_bin_ranks_uniform,
    expr_bin_ranks_uniform_labeled,
    |_| None
);

#[cfg(feature = "real-polars")]
fn materialize_series_expression(
    descriptor: String,
) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::{DataFrame, IntoLazy};

    let output = DataFrame::empty()
        .lazy()
        .select([compile_one(&descriptor)?])
        .collect()?;
    let column = output.select_at_idx(0).ok_or_else(|| {
        TerlanPolarsError::new(
            "invalid_expression",
            "Series expression produced no output column",
        )
    })?;
    Ok(crate::TerlanPolarsSeries {
        inner: column.as_materialized_series().clone(),
    })
}

#[cfg(feature = "real-polars")]
fn series_input_expression(input: &crate::TerlanPolarsSeries) -> Result<String, TerlanPolarsError> {
    expr_series(input)
}

#[cfg(feature = "real-polars")]
macro_rules! series_interval_bin_builders {
    ($plain:ident, $labeled:ident, $expr_plain:ident, $expr_labeled:ident, $ty:ty) => {
        /// Bins one Series at explicit breakpoints and returns integer indices.
        pub fn $plain(
            input: &crate::TerlanPolarsSeries,
            breaks: &[$ty],
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
            let input = series_input_expression(input)?;
            materialize_series_expression($expr_plain(
                &input,
                breaks,
                right_closed,
                include_intervals,
            )?)
        }

        /// Bins one Series at explicit breakpoints and returns caller labels.
        pub fn $labeled(
            input: &crate::TerlanPolarsSeries,
            breaks: &[$ty],
            labels: &[String],
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
            let input = series_input_expression(input)?;
            materialize_series_expression($expr_labeled(
                &input,
                breaks,
                labels,
                right_closed,
                include_intervals,
            )?)
        }
    };
}

#[cfg(feature = "real-polars")]
series_interval_bin_builders!(
    series_bin_intervals,
    series_bin_intervals_labeled,
    expr_bin_intervals,
    expr_bin_intervals_labeled,
    f64
);
#[cfg(feature = "real-polars")]
series_interval_bin_builders!(
    series_bin_intervals_int,
    series_bin_intervals_int_labeled,
    expr_bin_intervals_int,
    expr_bin_intervals_int_labeled,
    i64
);
#[cfg(feature = "real-polars")]
series_interval_bin_builders!(
    series_bin_intervals_string,
    series_bin_intervals_string_labeled,
    expr_bin_intervals_string,
    expr_bin_intervals_string_labeled,
    String
);
#[cfg(feature = "real-polars")]
series_interval_bin_builders!(
    series_bin_intervals_bool,
    series_bin_intervals_bool_labeled,
    expr_bin_intervals_bool,
    expr_bin_intervals_bool_labeled,
    bool
);

#[cfg(feature = "real-polars")]
/// Bins one Series using an arbitrary orderable breakpoint Series.
pub fn series_bin_intervals_series(
    input: &crate::TerlanPolarsSeries,
    breaks: &crate::TerlanPolarsSeries,
    right_closed: bool,
    include_intervals: bool,
) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
    let input = series_input_expression(input)?;
    materialize_series_expression(expr_bin_intervals_series(
        &input,
        breaks,
        right_closed,
        include_intervals,
    )?)
}

#[cfg(feature = "real-polars")]
/// Bins one Series using an arbitrary breakpoint Series and caller labels.
pub fn series_bin_intervals_series_labeled(
    input: &crate::TerlanPolarsSeries,
    breaks: &crate::TerlanPolarsSeries,
    labels: &[String],
    right_closed: bool,
    include_intervals: bool,
) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
    let input = series_input_expression(input)?;
    materialize_series_expression(expr_bin_intervals_series_labeled(
        &input,
        breaks,
        labels,
        right_closed,
        include_intervals,
    )?)
}

#[cfg(feature = "real-polars")]
/// Bins one numeric Series into equal-width intervals.
pub fn series_bin_intervals_uniform(
    input: &crate::TerlanPolarsSeries,
    bins: i64,
    right_closed: bool,
    include_intervals: bool,
) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
    let input = series_input_expression(input)?;
    materialize_series_expression(expr_bin_intervals_uniform(
        &input,
        bins,
        right_closed,
        include_intervals,
    )?)
}

#[cfg(feature = "real-polars")]
/// Bins one numeric Series into labeled equal-width intervals.
pub fn series_bin_intervals_uniform_labeled(
    input: &crate::TerlanPolarsSeries,
    bins: i64,
    labels: &[String],
    right_closed: bool,
    include_intervals: bool,
) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
    let input = series_input_expression(input)?;
    materialize_series_expression(expr_bin_intervals_uniform_labeled(
        &input,
        bins,
        labels,
        right_closed,
        include_intervals,
    )?)
}

#[cfg(feature = "real-polars")]
macro_rules! series_fraction_bin_builders {
    ($plain:ident, $labeled:ident, $uniform:ident, $uniform_labeled:ident, $expr_plain:ident, $expr_labeled:ident, $expr_uniform:ident, $expr_uniform_labeled:ident) => {
        /// Bins one Series at explicit cumulative fractions and returns indices.
        pub fn $plain(
            input: &crate::TerlanPolarsSeries,
            fractions: &[f64],
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
            let input = series_input_expression(input)?;
            materialize_series_expression($expr_plain(
                &input,
                fractions,
                right_closed,
                include_intervals,
            )?)
        }

        /// Bins one Series at explicit cumulative fractions and returns labels.
        pub fn $labeled(
            input: &crate::TerlanPolarsSeries,
            fractions: &[f64],
            labels: &[String],
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
            let input = series_input_expression(input)?;
            materialize_series_expression($expr_labeled(
                &input,
                fractions,
                labels,
                right_closed,
                include_intervals,
            )?)
        }

        /// Bins one Series into a positive number of uniform bins.
        pub fn $uniform(
            input: &crate::TerlanPolarsSeries,
            bins: i64,
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
            let input = series_input_expression(input)?;
            materialize_series_expression($expr_uniform(
                &input,
                bins,
                right_closed,
                include_intervals,
            )?)
        }

        /// Bins one Series into uniform bins and returns caller labels.
        pub fn $uniform_labeled(
            input: &crate::TerlanPolarsSeries,
            bins: i64,
            labels: &[String],
            right_closed: bool,
            include_intervals: bool,
        ) -> Result<crate::TerlanPolarsSeries, TerlanPolarsError> {
            let input = series_input_expression(input)?;
            materialize_series_expression($expr_uniform_labeled(
                &input,
                bins,
                labels,
                right_closed,
                include_intervals,
            )?)
        }
    };
}

#[cfg(feature = "real-polars")]
series_fraction_bin_builders!(
    series_bin_quantiles,
    series_bin_quantiles_labeled,
    series_bin_quantiles_uniform,
    series_bin_quantiles_uniform_labeled,
    expr_bin_quantiles,
    expr_bin_quantiles_labeled,
    expr_bin_quantiles_uniform,
    expr_bin_quantiles_uniform_labeled
);
#[cfg(feature = "real-polars")]
series_fraction_bin_builders!(
    series_bin_ranks,
    series_bin_ranks_labeled,
    series_bin_ranks_uniform,
    series_bin_ranks_uniform_labeled,
    expr_bin_ranks,
    expr_bin_ranks_labeled,
    expr_bin_ranks_uniform,
    expr_bin_ranks_uniform_labeled
);

/// Builds a Polars expression descriptor that applies the dot operation.
pub fn expr_dot(input: &str, other: &str) -> Result<String, TerlanPolarsError> {
    binary(input, other, BinaryOp::Dot)
}

fn statistic(
    input: &str,
    by: Option<&str>,
    kind: StatisticKind,
    ddof: Option<i64>,
) -> Result<String, TerlanPolarsError> {
    let ddof = ddof
        .map(|value| {
            u8::try_from(value).map_err(|_| {
                TerlanPolarsError::new(
                    "invalid_expression",
                    "statistics degrees of freedom must be between 0 and 255",
                )
            })
        })
        .transpose()?;
    encode(&ExprNode::Statistic {
        input: Box::new(decode(input)?),
        by: by.map(decode).transpose()?.map(Box::new),
        kind,
        ddof,
    })
}

/// Builds a Polars expression descriptor that applies the std operation.
pub fn expr_std(input: &str, ddof: i64) -> Result<String, TerlanPolarsError> {
    statistic(input, None, StatisticKind::Std, Some(ddof))
}

/// Builds a Polars expression descriptor that applies the var operation.
pub fn expr_var(input: &str, ddof: i64) -> Result<String, TerlanPolarsError> {
    statistic(input, None, StatisticKind::Var, Some(ddof))
}

/// Builds a Polars expression descriptor that applies the min operation.
pub fn expr_min(input: &str) -> Result<String, TerlanPolarsError> {
    statistic(input, None, StatisticKind::Min, None)
}

/// Builds a Polars expression descriptor that applies the median operation.
pub fn expr_median(input: &str) -> Result<String, TerlanPolarsError> {
    statistic(input, None, StatisticKind::Median, None)
}

/// Builds a Polars expression descriptor that applies the min by operation.
pub fn expr_min_by(input: &str, by: &str) -> Result<String, TerlanPolarsError> {
    statistic(input, Some(by), StatisticKind::MinBy, None)
}

/// Builds a Polars expression descriptor that applies the max by operation.
pub fn expr_max_by(input: &str, by: &str) -> Result<String, TerlanPolarsError> {
    statistic(input, Some(by), StatisticKind::MaxBy, None)
}

/// Builds a Polars expression descriptor that applies the nan min operation.
pub fn expr_nan_min(input: &str) -> Result<String, TerlanPolarsError> {
    statistic(input, None, StatisticKind::NanMin, None)
}

/// Builds a Polars expression descriptor that applies the nan max operation.
pub fn expr_nan_max(input: &str) -> Result<String, TerlanPolarsError> {
    statistic(input, None, StatisticKind::NanMax, None)
}

fn cumulative(
    input: &str,
    kind: CumulativeKind,
    reverse: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Cumulative {
        input: Box::new(decode(input)?),
        kind,
        reverse,
    })
}

/// Builds a Polars expression descriptor that applies the cum count operation.
pub fn expr_cum_count(input: &str, reverse: bool) -> Result<String, TerlanPolarsError> {
    cumulative(input, CumulativeKind::Count, reverse)
}

/// Builds a Polars expression descriptor that applies the cum sum operation.
pub fn expr_cum_sum(input: &str, reverse: bool) -> Result<String, TerlanPolarsError> {
    cumulative(input, CumulativeKind::Sum, reverse)
}

/// Builds a Polars expression descriptor that applies the cum product operation.
pub fn expr_cum_product(input: &str, reverse: bool) -> Result<String, TerlanPolarsError> {
    cumulative(input, CumulativeKind::Product, reverse)
}

/// Builds a Polars expression descriptor that applies the cum min operation.
pub fn expr_cum_min(input: &str, reverse: bool) -> Result<String, TerlanPolarsError> {
    cumulative(input, CumulativeKind::Min, reverse)
}

/// Builds a Polars expression descriptor that applies the cum max operation.
pub fn expr_cum_max(input: &str, reverse: bool) -> Result<String, TerlanPolarsError> {
    cumulative(input, CumulativeKind::Max, reverse)
}

fn rolling(
    input: &str,
    kind: RollingKind,
    window_size: i64,
    min_samples: i64,
    center: bool,
    weights: Option<&[f64]>,
) -> Result<String, TerlanPolarsError> {
    let window_size = usize::try_from(window_size).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "rolling window size must be greater than zero",
        )
    })?;
    let min_samples = usize::try_from(min_samples).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "rolling minimum sample count cannot be negative",
        )
    })?;
    if window_size == 0 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "rolling window size must be greater than zero",
        ));
    }
    if min_samples > window_size {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "rolling minimum sample count cannot exceed the window size",
        ));
    }
    let weights = weights
        .map(|weights| {
            if weights.len() != window_size {
                return Err(TerlanPolarsError::new(
                    "invalid_expression",
                    "rolling weights must contain exactly one value per window position",
                ));
            }
            if weights.iter().any(|weight| !weight.is_finite()) {
                return Err(TerlanPolarsError::new(
                    "invalid_expression",
                    "rolling weights must contain only finite values",
                ));
            }
            Ok(weights.to_vec())
        })
        .transpose()?;
    encode(&ExprNode::Rolling {
        input: Box::new(decode(input)?),
        kind,
        window_size,
        min_samples,
        center,
        weights,
    })
}

/// Applies a safe declarative one-argument expression UDF to every rolling Series window.
pub fn expr_rolling_map(
    input: &str,
    udf: &str,
    window_size: i64,
    min_samples: i64,
    center: bool,
) -> Result<String, TerlanPolarsError> {
    let window_size = usize::try_from(window_size).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "rolling map window size must be greater than zero",
        )
    })?;
    let min_samples = usize::try_from(min_samples).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "rolling map minimum sample count cannot be negative",
        )
    })?;
    if window_size == 0 || min_samples > window_size {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "rolling map requires a positive window and no more minimum samples than window values",
        ));
    }
    let validation_argument = expr_null()?;
    crate::apply_expression_udf(udf, &[validation_argument])?;
    encode(&ExprNode::RollingMap {
        input: Box::new(decode(input)?),
        udf: udf.into(),
        window_size,
        min_samples,
        center,
    })
}

/// Applies an aggregation expression over temporal rolling groups.
pub fn expr_rolling(
    input: &str,
    index_column: &str,
    period: &str,
    offset: &str,
    closed: &str,
) -> Result<String, TerlanPolarsError> {
    if period.trim().is_empty() || offset.trim().is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "rolling period and offset must be non-empty duration strings",
        ));
    }
    encode(&ExprNode::RollingGroup {
        input: Box::new(decode(input)?),
        index_column: Box::new(decode(index_column)?),
        period: period.into(),
        offset: offset.into(),
        closed: parse_closed_interval(closed)?,
    })
}

macro_rules! rolling_expression {
    ($name:ident, $kind:ident) => {
        /// Builds a fixed-window rolling expression.
        pub fn $name(
            input: &str,
            window_size: i64,
            min_samples: i64,
            center: bool,
        ) -> Result<String, TerlanPolarsError> {
            rolling(
                input,
                RollingKind::$kind,
                window_size,
                min_samples,
                center,
                None,
            )
        }
    };
}

rolling_expression!(expr_rolling_min, Min);
rolling_expression!(expr_rolling_max, Max);
rolling_expression!(expr_rolling_mean, Mean);
rolling_expression!(expr_rolling_sum, Sum);
rolling_expression!(expr_rolling_median, Median);
rolling_expression!(expr_rolling_var, Var);
rolling_expression!(expr_rolling_std, Std);

macro_rules! weighted_rolling_expression {
    ($name:ident, $kind:ident) => {
        /// Builds a weighted fixed-window rolling expression.
        pub fn $name(
            input: &str,
            window_size: i64,
            min_samples: i64,
            center: bool,
            weights: &[f64],
        ) -> Result<String, TerlanPolarsError> {
            rolling(
                input,
                RollingKind::$kind,
                window_size,
                min_samples,
                center,
                Some(weights),
            )
        }
    };
}

weighted_rolling_expression!(expr_rolling_min_weighted, Min);
weighted_rolling_expression!(expr_rolling_max_weighted, Max);
weighted_rolling_expression!(expr_rolling_mean_weighted, Mean);
weighted_rolling_expression!(expr_rolling_sum_weighted, Sum);
weighted_rolling_expression!(expr_rolling_median_weighted, Median);
weighted_rolling_expression!(expr_rolling_var_weighted, Var);
weighted_rolling_expression!(expr_rolling_std_weighted, Std);

/// Builds a Polars expression descriptor that applies the rolling quantile operation.
pub fn expr_rolling_quantile(
    input: &str,
    window_size: i64,
    min_samples: i64,
    center: bool,
    probability: f64,
    method: &str,
) -> Result<String, TerlanPolarsError> {
    if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "rolling quantile probability must be finite and between zero and one",
        ));
    }
    let method = match method {
        "nearest" => RollingQuantileMethod::Nearest,
        "lower" => RollingQuantileMethod::Lower,
        "higher" => RollingQuantileMethod::Higher,
        "midpoint" => RollingQuantileMethod::Midpoint,
        "linear" => RollingQuantileMethod::Linear,
        "equiprobable" => RollingQuantileMethod::Equiprobable,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "rolling quantile method must be nearest, lower, higher, midpoint, linear, or equiprobable",
            ));
        }
    };
    rolling(
        input,
        RollingKind::Quantile {
            probability,
            method,
        },
        window_size,
        min_samples,
        center,
        None,
    )
}

/// Builds a fixed-window weighted quantile expression.
pub fn expr_rolling_quantile_weighted(
    input: &str,
    window_size: i64,
    min_samples: i64,
    center: bool,
    weights: &[f64],
    probability: f64,
    method: &str,
) -> Result<String, TerlanPolarsError> {
    if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "rolling quantile probability must be finite and between zero and one",
        ));
    }
    let method = match method {
        "nearest" => RollingQuantileMethod::Nearest,
        "lower" => RollingQuantileMethod::Lower,
        "higher" => RollingQuantileMethod::Higher,
        "midpoint" => RollingQuantileMethod::Midpoint,
        "linear" => RollingQuantileMethod::Linear,
        "equiprobable" => RollingQuantileMethod::Equiprobable,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "rolling quantile method must be nearest, lower, higher, midpoint, linear, or equiprobable",
            ));
        }
    };
    if weights.iter().sum::<f64>() == 0.0 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "weighted rolling quantile requires weights with a nonzero sum",
        ));
    }
    rolling(
        input,
        RollingKind::Quantile {
            probability,
            method,
        },
        window_size,
        min_samples,
        center,
        Some(weights),
    )
}

/// Builds a Polars expression descriptor that applies the rolling rank operation.
pub fn expr_rolling_rank(
    input: &str,
    window_size: i64,
    min_samples: i64,
    center: bool,
    method: &str,
    seed: i64,
) -> Result<String, TerlanPolarsError> {
    let method = match method {
        "average" => RollingRankKind::Average,
        "min" => RollingRankKind::Min,
        "max" => RollingRankKind::Max,
        "dense" => RollingRankKind::Dense,
        "random" => RollingRankKind::Random,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "rolling rank method must be average, min, max, dense, or random",
            ));
        }
    };
    let seed = u64::try_from(seed).map_err(|_| {
        TerlanPolarsError::new("invalid_expression", "rolling rank seed cannot be negative")
    })?;
    rolling(
        input,
        RollingKind::Rank { method, seed },
        window_size,
        min_samples,
        center,
        None,
    )
}

/// Builds a Polars expression descriptor that applies the rolling skew operation.
pub fn expr_rolling_skew(
    input: &str,
    window_size: i64,
    min_samples: i64,
    center: bool,
    bias: bool,
) -> Result<String, TerlanPolarsError> {
    rolling(
        input,
        RollingKind::Skew { bias },
        window_size,
        min_samples,
        center,
        None,
    )
}

/// Builds a Polars expression descriptor that applies the rolling kurtosis operation.
pub fn expr_rolling_kurtosis(
    input: &str,
    window_size: i64,
    min_samples: i64,
    center: bool,
    fisher: bool,
    bias: bool,
) -> Result<String, TerlanPolarsError> {
    rolling(
        input,
        RollingKind::Kurtosis { fisher, bias },
        window_size,
        min_samples,
        center,
        None,
    )
}

fn rolling_by(
    input: &str,
    by: &str,
    kind: RollingKind,
    window_size: &str,
    min_samples: i64,
    closed: &str,
) -> Result<String, TerlanPolarsError> {
    if window_size.trim().is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "rolling-by window duration cannot be empty",
        ));
    }
    let min_samples = usize::try_from(min_samples).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "rolling-by minimum sample count cannot be negative",
        )
    })?;
    let closed = match closed {
        "both" => ClosedIntervalKind::Both,
        "left" => ClosedIntervalKind::Left,
        "right" => ClosedIntervalKind::Right,
        "none" | "neither" => ClosedIntervalKind::None,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "rolling-by closed window must be both, left, right, or none",
            ));
        }
    };
    if matches!(kind, RollingKind::Rank { .. })
        && !matches!(closed, ClosedIntervalKind::Right | ClosedIntervalKind::Both)
    {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "rolling rank-by windows must be closed on the right or both sides",
        ));
    }
    encode(&ExprNode::RollingBy {
        input: Box::new(decode(input)?),
        by: Box::new(decode(by)?),
        kind,
        window_size: window_size.into(),
        min_samples,
        closed,
    })
}

macro_rules! rolling_by_expression {
    ($name:ident, $kind:ident) => {
        /// Builds a rolling expression over a temporal index.
        pub fn $name(
            input: &str,
            by: &str,
            window_size: &str,
            min_samples: i64,
            closed: &str,
        ) -> Result<String, TerlanPolarsError> {
            rolling_by(
                input,
                by,
                RollingKind::$kind,
                window_size,
                min_samples,
                closed,
            )
        }
    };
}

rolling_by_expression!(expr_rolling_min_by, Min);
rolling_by_expression!(expr_rolling_max_by, Max);
rolling_by_expression!(expr_rolling_mean_by, Mean);
rolling_by_expression!(expr_rolling_sum_by, Sum);
rolling_by_expression!(expr_rolling_median_by, Median);
rolling_by_expression!(expr_rolling_var_by, Var);
rolling_by_expression!(expr_rolling_std_by, Std);

/// Builds a Polars expression descriptor that applies the rolling quantile by operation.
pub fn expr_rolling_quantile_by(
    input: &str,
    by: &str,
    window_size: &str,
    min_samples: i64,
    closed: &str,
    probability: f64,
    method: &str,
) -> Result<String, TerlanPolarsError> {
    if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "rolling quantile-by probability must be finite and between zero and one",
        ));
    }
    let method = match method {
        "nearest" => RollingQuantileMethod::Nearest,
        "lower" => RollingQuantileMethod::Lower,
        "higher" => RollingQuantileMethod::Higher,
        "midpoint" => RollingQuantileMethod::Midpoint,
        "linear" => RollingQuantileMethod::Linear,
        "equiprobable" => RollingQuantileMethod::Equiprobable,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "rolling quantile-by method must be nearest, lower, higher, midpoint, linear, or equiprobable",
            ));
        }
    };
    rolling_by(
        input,
        by,
        RollingKind::Quantile {
            probability,
            method,
        },
        window_size,
        min_samples,
        closed,
    )
}

/// Builds a Polars expression descriptor that applies the rolling rank by operation.
pub fn expr_rolling_rank_by(
    input: &str,
    by: &str,
    window_size: &str,
    min_samples: i64,
    closed: &str,
    method: &str,
    seed: i64,
) -> Result<String, TerlanPolarsError> {
    let method = match method {
        "average" => RollingRankKind::Average,
        "min" => RollingRankKind::Min,
        "max" => RollingRankKind::Max,
        "dense" => RollingRankKind::Dense,
        "random" => RollingRankKind::Random,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "rolling rank-by method must be average, min, max, dense, or random",
            ));
        }
    };
    let seed = u64::try_from(seed).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "rolling rank-by seed cannot be negative",
        )
    })?;
    rolling_by(
        input,
        by,
        RollingKind::Rank { method, seed },
        window_size,
        min_samples,
        closed,
    )
}

fn histogram(
    input: &str,
    bins: Option<&str>,
    bin_count: Option<i64>,
    include_category: bool,
    include_breakpoint: bool,
) -> Result<String, TerlanPolarsError> {
    let bin_count = bin_count
        .map(|value| {
            usize::try_from(value).map_err(|_| {
                TerlanPolarsError::new(
                    "invalid_expression",
                    "histogram bin count cannot be negative",
                )
            })
        })
        .transpose()?;
    if bin_count == Some(0) {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "histogram bin count must be greater than zero",
        ));
    }
    encode(&ExprNode::Histogram {
        input: Box::new(decode(input)?),
        bins: bins.map(decode).transpose()?.map(Box::new),
        bin_count,
        include_category,
        include_breakpoint,
    })
}

/// Builds a Polars expression descriptor that applies the histogram auto operation.
pub fn expr_histogram_auto(
    input: &str,
    include_category: bool,
    include_breakpoint: bool,
) -> Result<String, TerlanPolarsError> {
    histogram(input, None, None, include_category, include_breakpoint)
}

/// Builds a Polars expression descriptor that applies the histogram count operation.
pub fn expr_histogram_count(
    input: &str,
    bin_count: i64,
    include_category: bool,
    include_breakpoint: bool,
) -> Result<String, TerlanPolarsError> {
    histogram(
        input,
        None,
        Some(bin_count),
        include_category,
        include_breakpoint,
    )
}

/// Builds a Polars expression descriptor that applies the histogram bins operation.
pub fn expr_histogram_bins(
    input: &str,
    bins: &str,
    include_category: bool,
    include_breakpoint: bool,
) -> Result<String, TerlanPolarsError> {
    histogram(
        input,
        Some(bins),
        None,
        include_category,
        include_breakpoint,
    )
}

fn decode_many(values: &[String]) -> Result<Vec<ExprNode>, TerlanPolarsError> {
    values.iter().map(|value| decode(value)).collect()
}

fn fold_expr(
    initial: &str,
    inputs: &[String],
    kind: FoldKind,
) -> Result<String, TerlanPolarsError> {
    if inputs.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "fold requires at least one expression",
        ));
    }
    encode(&ExprNode::Fold {
        kind,
        initial: Some(Box::new(decode(initial)?)),
        inputs: decode_many(inputs)?,
        separator: None,
    })
}

/// Builds a Polars expression descriptor that applies the fold sum operation.
pub fn expr_fold_sum(initial: &str, inputs: &[String]) -> Result<String, TerlanPolarsError> {
    fold_expr(initial, inputs, FoldKind::Sum)
}

/// Builds a Polars expression descriptor that applies the fold product operation.
pub fn expr_fold_product(initial: &str, inputs: &[String]) -> Result<String, TerlanPolarsError> {
    fold_expr(initial, inputs, FoldKind::Product)
}

/// Builds a Polars expression descriptor that applies the sum horizontal operation.
pub fn expr_sum_horizontal(inputs: &[String]) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Fold {
        kind: FoldKind::HorizontalSum,
        initial: None,
        inputs: decode_many(inputs)?,
        separator: None,
    })
}

/// Builds a Polars expression descriptor that applies the all horizontal operation.
pub fn expr_all_horizontal(inputs: &[String]) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Fold {
        kind: FoldKind::HorizontalAll,
        initial: None,
        inputs: decode_many(inputs)?,
        separator: None,
    })
}

/// Builds a Polars expression descriptor that applies the any horizontal operation.
pub fn expr_any_horizontal(inputs: &[String]) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Fold {
        kind: FoldKind::HorizontalAny,
        initial: None,
        inputs: decode_many(inputs)?,
        separator: None,
    })
}

/// Builds a Polars expression descriptor that applies the concat string operation.
pub fn expr_concat_string(inputs: &[String], separator: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Fold {
        kind: FoldKind::ConcatString,
        initial: None,
        inputs: decode_many(inputs)?,
        separator: Some(separator.into()),
    })
}

/// Builds a Polars expression descriptor that applies the first operation.
pub fn expr_first(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::First)
}
/// Builds a Polars expression descriptor that applies the last operation.
pub fn expr_last(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Last)
}
/// Builds a Polars expression descriptor that applies the sort ascending operation.
pub fn expr_sort_ascending(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::SortAscending)
}
/// Builds a Polars expression descriptor that applies the rank dense descending operation.
pub fn expr_rank_dense_descending(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::RankDenseDescending)
}
/// Builds a Polars expression descriptor that applies the explode operation.
pub fn expr_explode(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Explode)
}

fn random_seed(seed: i64) -> Result<u64, TerlanPolarsError> {
    u64::try_from(seed)
        .map_err(|_| TerlanPolarsError::new("invalid_expression", "random seed cannot be negative"))
}

fn random_expression(
    input: &str,
    amount: Option<&str>,
    fraction: bool,
    with_replacement: bool,
    shuffle: bool,
    seed: Option<i64>,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Random {
        input: Box::new(decode(input)?),
        amount: amount.map(decode).transpose()?.map(Box::new),
        fraction,
        with_replacement,
        shuffle,
        seed: seed.map(random_seed).transpose()?,
    })
}

/// Builds a Polars expression descriptor that applies the shuffle operation.
pub fn expr_shuffle(input: &str) -> Result<String, TerlanPolarsError> {
    random_expression(input, None, false, false, false, None)
}

/// Builds a Polars expression descriptor that applies the shuffle seeded operation.
pub fn expr_shuffle_seeded(input: &str, seed: i64) -> Result<String, TerlanPolarsError> {
    random_expression(input, None, false, false, false, Some(seed))
}

/// Builds a Polars expression descriptor that applies the sample n operation.
pub fn expr_sample_n(
    input: &str,
    amount: &str,
    with_replacement: bool,
    shuffle: bool,
) -> Result<String, TerlanPolarsError> {
    random_expression(input, Some(amount), false, with_replacement, shuffle, None)
}

/// Builds a Polars expression descriptor that applies the sample n seeded operation.
pub fn expr_sample_n_seeded(
    input: &str,
    amount: &str,
    with_replacement: bool,
    shuffle: bool,
    seed: i64,
) -> Result<String, TerlanPolarsError> {
    random_expression(
        input,
        Some(amount),
        false,
        with_replacement,
        shuffle,
        Some(seed),
    )
}

/// Builds a Polars expression descriptor that applies the sample fraction operation.
pub fn expr_sample_fraction(
    input: &str,
    amount: &str,
    with_replacement: bool,
    shuffle: bool,
) -> Result<String, TerlanPolarsError> {
    random_expression(input, Some(amount), true, with_replacement, shuffle, None)
}

/// Builds a Polars expression descriptor that applies the sample fraction seeded operation.
pub fn expr_sample_fraction_seeded(
    input: &str,
    amount: &str,
    with_replacement: bool,
    shuffle: bool,
    seed: i64,
) -> Result<String, TerlanPolarsError> {
    random_expression(
        input,
        Some(amount),
        true,
        with_replacement,
        shuffle,
        Some(seed),
    )
}

/// Hashes expression values using the Polars 2.0 single-seed model.
pub fn expr_hash(input: &str, seed: i64) -> Result<String, TerlanPolarsError> {
    let seed = u64::try_from(seed).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "expression hash seed cannot be negative",
        )
    })?;
    encode(&ExprNode::Hash {
        input: Box::new(decode(input)?),
        seed,
    })
}

/// Builds a Polars expression descriptor that applies the bitwise count ones operation.
pub fn expr_bitwise_count_ones(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BitwiseCountOnes)
}

/// Builds a Polars expression descriptor that applies the bitwise count zeros operation.
pub fn expr_bitwise_count_zeros(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BitwiseCountZeros)
}

/// Builds a Polars expression descriptor that applies the bitwise leading ones operation.
pub fn expr_bitwise_leading_ones(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BitwiseLeadingOnes)
}

/// Builds a Polars expression descriptor that applies the bitwise leading zeros operation.
pub fn expr_bitwise_leading_zeros(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BitwiseLeadingZeros)
}

/// Builds a Polars expression descriptor that applies the bitwise trailing ones operation.
pub fn expr_bitwise_trailing_ones(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BitwiseTrailingOnes)
}

/// Builds a Polars expression descriptor that applies the bitwise trailing zeros operation.
pub fn expr_bitwise_trailing_zeros(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BitwiseTrailingZeros)
}

/// Builds a Polars expression descriptor that applies the bitwise and operation.
pub fn expr_bitwise_and(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BitwiseAnd)
}

/// Builds a Polars expression descriptor that applies the bitwise or operation.
pub fn expr_bitwise_or(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BitwiseOr)
}

/// Builds a Polars expression descriptor that applies the bitwise xor operation.
pub fn expr_bitwise_xor(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BitwiseXor)
}

/// Builds a Polars expression descriptor that applies the list len operation.
pub fn expr_list_len(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListLen)
}

/// Converts a List of `{key, value}` Struct entries to a Map.
pub fn expr_list_to_map(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListToMap)
}

/// Converts each Map to its List of `{key, value}` Struct entries.
pub fn expr_map_entries(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::MapEntries)
}

/// Returns each Map's keys in entry order.
pub fn expr_map_keys(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::MapKeys)
}

/// Returns each Map's values in entry order.
pub fn expr_map_values(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::MapValues)
}

/// Returns the number of entries in each Map.
pub fn expr_map_len(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::MapLen)
}

/// Tests whether each Map contains the requested key.
pub fn expr_map_contains_key(input: &str, key: &str) -> Result<String, TerlanPolarsError> {
    binary(input, key, BinaryOp::MapContainsKey)
}

/// Looks up the requested key in each Map.
pub fn expr_map_get(input: &str, key: &str) -> Result<String, TerlanPolarsError> {
    binary(input, key, BinaryOp::MapGet)
}

/// Builds a Polars expression descriptor that applies the list sum operation.
pub fn expr_list_sum(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListSum)
}

/// Builds a Polars expression descriptor that applies the list mean operation.
pub fn expr_list_mean(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListMean)
}

/// Builds a Polars expression descriptor that applies the list min operation.
pub fn expr_list_min(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListMin)
}

/// Builds a Polars expression descriptor that applies the list max operation.
pub fn expr_list_max(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListMax)
}

/// Builds a Polars expression descriptor that applies the list sort operation.
pub fn expr_list_sort(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListSort)
}

/// Builds a Polars expression descriptor that applies the list get operation.
pub fn expr_list_get(input: &str, index: &str) -> Result<String, TerlanPolarsError> {
    binary(input, index, BinaryOp::ListGet)
}

/// Builds a Polars expression descriptor that applies the list contains operation.
pub fn expr_list_contains(input: &str, value: &str) -> Result<String, TerlanPolarsError> {
    binary(input, value, BinaryOp::ListContains)
}

fn list_statistic(input: &str, ddof: i64, variance: bool) -> Result<String, TerlanPolarsError> {
    let ddof = u8::try_from(ddof).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "list statistic degrees of freedom must be between 0 and 255",
        )
    })?;
    encode(&ExprNode::ListStatistic {
        input: Box::new(decode(input)?),
        ddof,
        variance,
    })
}

/// Builds a Polars expression descriptor that applies the list std operation.
pub fn expr_list_std(input: &str, ddof: i64) -> Result<String, TerlanPolarsError> {
    list_statistic(input, ddof, false)
}

/// Builds a Polars expression descriptor that applies the list var operation.
pub fn expr_list_var(input: &str, ddof: i64) -> Result<String, TerlanPolarsError> {
    list_statistic(input, ddof, true)
}

/// Builds a Polars expression descriptor that applies the list median operation.
pub fn expr_list_median(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListMedian)
}

/// Builds a Polars expression descriptor that applies the list first operation.
pub fn expr_list_first(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListFirst)
}

/// Builds a Polars expression descriptor that applies the list last operation.
pub fn expr_list_last(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListLast)
}

/// Builds a Polars expression descriptor that applies the list arg min operation.
pub fn expr_list_arg_min(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListArgMin)
}

/// Builds a Polars expression descriptor that applies the list arg max operation.
pub fn expr_list_arg_max(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListArgMax)
}

/// Builds a Polars expression descriptor that applies the list join operation.
pub fn expr_list_join(
    input: &str,
    separator: &str,
    ignore_nulls: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ListJoin {
        input: Box::new(decode(input)?),
        separator: Box::new(decode(separator)?),
        ignore_nulls,
    })
}

/// Builds a Polars expression descriptor that applies the list shift operation.
pub fn expr_list_shift(input: &str, periods: &str) -> Result<String, TerlanPolarsError> {
    binary(input, periods, BinaryOp::ListShift)
}

fn list_window(
    input: &str,
    first: &str,
    second: Option<&str>,
    kind: ListWindowKind,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ListWindow {
        input: Box::new(decode(input)?),
        first: Box::new(decode(first)?),
        second: second.map(decode).transpose()?.map(Box::new),
        kind,
    })
}

/// Builds a Polars expression descriptor that applies the list slice operation.
pub fn expr_list_slice(
    input: &str,
    offset: &str,
    length: &str,
) -> Result<String, TerlanPolarsError> {
    list_window(input, offset, Some(length), ListWindowKind::Slice)
}

/// Builds a Polars expression descriptor that applies the list head operation.
pub fn expr_list_head(input: &str, length: &str) -> Result<String, TerlanPolarsError> {
    list_window(input, length, None, ListWindowKind::Head)
}

/// Builds a Polars expression descriptor that applies the list tail operation.
pub fn expr_list_tail(input: &str, length: &str) -> Result<String, TerlanPolarsError> {
    list_window(input, length, None, ListWindowKind::Tail)
}

/// Builds a Polars expression descriptor that applies the list to array operation.
pub fn expr_list_to_array(input: &str, width: i64) -> Result<String, TerlanPolarsError> {
    let width = usize::try_from(width).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "List-to-Array width cannot be negative",
        )
    })?;
    if width == 0 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "List-to-Array width must be greater than zero",
        ));
    }
    encode(&ExprNode::ListToArray {
        input: Box::new(decode(input)?),
        width,
    })
}

/// Builds a Polars expression descriptor that applies the list eval operation.
pub fn expr_list_eval(input: &str, evaluation: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ListEval {
        input: Box::new(decode(input)?),
        evaluation: Box::new(decode(evaluation)?),
        aggregate: false,
    })
}

/// Builds a Polars expression descriptor that applies the list agg operation.
pub fn expr_list_agg(input: &str, evaluation: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ListEval {
        input: Box::new(decode(input)?),
        evaluation: Box::new(decode(evaluation)?),
        aggregate: true,
    })
}

/// Builds a Polars expression descriptor that applies the list sort with operation.
pub fn expr_list_sort_with(
    input: &str,
    descending: bool,
    nulls_last: bool,
    maintain_order: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ListSortOptions {
        input: Box::new(decode(input)?),
        descending,
        nulls_last,
        maintain_order,
    })
}

fn list_binary_options(
    input: &str,
    value: &str,
    kind: ListBinaryOptionsKind,
    option: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ListBinaryOptions {
        input: Box::new(decode(input)?),
        value: Box::new(decode(value)?),
        kind,
        option,
    })
}

/// Builds a Polars expression descriptor that applies the list get with operation.
pub fn expr_list_get_with(
    input: &str,
    index: &str,
    null_on_oob: bool,
) -> Result<String, TerlanPolarsError> {
    list_binary_options(input, index, ListBinaryOptionsKind::Get, null_on_oob)
}

/// Builds a Polars expression descriptor that applies the list contains with operation.
pub fn expr_list_contains_with(
    input: &str,
    value: &str,
    nulls_equal: bool,
) -> Result<String, TerlanPolarsError> {
    list_binary_options(input, value, ListBinaryOptionsKind::Contains, nulls_equal)
}

/// Builds a Polars expression descriptor that applies the list drop nulls operation.
pub fn expr_list_drop_nulls(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ListDropNulls)
}

fn list_sample(
    input: &str,
    amount: &str,
    fraction: bool,
    with_replacement: bool,
    shuffle: bool,
    seed: Option<i64>,
) -> Result<String, TerlanPolarsError> {
    let seed = seed
        .map(|value| {
            u64::try_from(value).map_err(|_| {
                TerlanPolarsError::new("invalid_expression", "List sample seed cannot be negative")
            })
        })
        .transpose()?;
    encode(&ExprNode::ListSample {
        input: Box::new(decode(input)?),
        amount: Box::new(decode(amount)?),
        fraction,
        with_replacement,
        shuffle,
        seed,
    })
}

/// Builds a Polars expression descriptor that applies the list sample n operation.
pub fn expr_list_sample_n(
    input: &str,
    amount: &str,
    with_replacement: bool,
    shuffle: bool,
) -> Result<String, TerlanPolarsError> {
    list_sample(input, amount, false, with_replacement, shuffle, None)
}

/// Builds a Polars expression descriptor that applies the list sample n seeded operation.
pub fn expr_list_sample_n_seeded(
    input: &str,
    amount: &str,
    with_replacement: bool,
    shuffle: bool,
    seed: i64,
) -> Result<String, TerlanPolarsError> {
    list_sample(input, amount, false, with_replacement, shuffle, Some(seed))
}

/// Builds a Polars expression descriptor that applies the list sample fraction operation.
pub fn expr_list_sample_fraction(
    input: &str,
    amount: &str,
    with_replacement: bool,
    shuffle: bool,
) -> Result<String, TerlanPolarsError> {
    list_sample(input, amount, true, with_replacement, shuffle, None)
}

/// Builds a Polars expression descriptor that applies the list sample fraction seeded operation.
pub fn expr_list_sample_fraction_seeded(
    input: &str,
    amount: &str,
    with_replacement: bool,
    shuffle: bool,
    seed: i64,
) -> Result<String, TerlanPolarsError> {
    list_sample(input, amount, true, with_replacement, shuffle, Some(seed))
}

/// Builds a Polars expression descriptor that applies the list gather operation.
pub fn expr_list_gather(
    input: &str,
    index: &str,
    null_on_oob: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ListGather {
        input: Box::new(decode(input)?),
        index: Box::new(decode(index)?),
        offset: None,
        null_on_oob,
    })
}

/// Builds a Polars expression descriptor that applies the list gather every operation.
pub fn expr_list_gather_every(
    input: &str,
    periods: &str,
    offset: &str,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ListGather {
        input: Box::new(decode(input)?),
        index: Box::new(decode(periods)?),
        offset: Some(Box::new(decode(offset)?)),
        null_on_oob: false,
    })
}

fn list_diff(input: &str, periods: i64, drop_nulls: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ListDiff {
        input: Box::new(decode(input)?),
        periods,
        drop_nulls,
    })
}

/// Builds a Polars expression descriptor that applies the list diff drop operation.
pub fn expr_list_diff_drop(input: &str, periods: i64) -> Result<String, TerlanPolarsError> {
    list_diff(input, periods, true)
}

/// Builds a Polars expression descriptor that applies the list diff ignore operation.
pub fn expr_list_diff_ignore(input: &str, periods: i64) -> Result<String, TerlanPolarsError> {
    list_diff(input, periods, false)
}

/// Builds a Polars expression descriptor that applies the list to struct operation.
pub fn expr_list_to_struct(input: &str, names: &[String]) -> Result<String, TerlanPolarsError> {
    if names.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "List-to-Struct conversion requires at least one field name",
        ));
    }
    encode(&ExprNode::ListToStruct {
        input: Box::new(decode(input)?),
        names: names.to_vec(),
    })
}

/// Builds a Polars expression descriptor that applies the list count matches operation.
pub fn expr_list_count_matches(input: &str, value: &str) -> Result<String, TerlanPolarsError> {
    binary(input, value, BinaryOp::ListCountMatches)
}

/// Builds a Polars expression descriptor that applies the list set union operation.
pub fn expr_list_set_union(input: &str, other: &str) -> Result<String, TerlanPolarsError> {
    binary(input, other, BinaryOp::ListSetUnion)
}

/// Builds a Polars expression descriptor that applies the list set difference operation.
pub fn expr_list_set_difference(input: &str, other: &str) -> Result<String, TerlanPolarsError> {
    binary(input, other, BinaryOp::ListSetDifference)
}

/// Builds a Polars expression descriptor that applies the list set intersection operation.
pub fn expr_list_set_intersection(input: &str, other: &str) -> Result<String, TerlanPolarsError> {
    binary(input, other, BinaryOp::ListSetIntersection)
}

/// Builds a Polars expression descriptor that applies the list set symmetric difference operation.
pub fn expr_list_set_symmetric_difference(
    input: &str,
    other: &str,
) -> Result<String, TerlanPolarsError> {
    binary(input, other, BinaryOp::ListSetSymmetricDifference)
}

/// Builds a Polars expression descriptor that applies the array len operation.
pub fn expr_array_len(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArrayLen)
}

/// Builds a Polars expression descriptor that applies the array sum operation.
pub fn expr_array_sum(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArraySum)
}

/// Builds a Polars expression descriptor that applies the array mean operation.
pub fn expr_array_mean(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArrayMean)
}

/// Builds a Polars expression descriptor that applies the array min operation.
pub fn expr_array_min(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArrayMin)
}

/// Builds a Polars expression descriptor that applies the array max operation.
pub fn expr_array_max(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArrayMax)
}

/// Builds a Polars expression descriptor that applies the array sort operation.
pub fn expr_array_sort(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArraySort)
}

/// Builds a Polars expression descriptor that applies the array to list operation.
pub fn expr_array_to_list(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArrayToList)
}

/// Builds a Polars expression descriptor that applies the array get operation.
pub fn expr_array_get(input: &str, index: &str) -> Result<String, TerlanPolarsError> {
    binary(input, index, BinaryOp::ArrayGet)
}

/// Builds a Polars expression descriptor that applies the array contains operation.
pub fn expr_array_contains(input: &str, value: &str) -> Result<String, TerlanPolarsError> {
    binary(input, value, BinaryOp::ArrayContains)
}

fn array_statistic(input: &str, ddof: i64, variance: bool) -> Result<String, TerlanPolarsError> {
    let ddof = u8::try_from(ddof).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "array statistic degrees of freedom must be between 0 and 255",
        )
    })?;
    encode(&ExprNode::ArrayStatistic {
        input: Box::new(decode(input)?),
        ddof,
        variance,
    })
}

/// Builds a Polars expression descriptor that applies the array std operation.
pub fn expr_array_std(input: &str, ddof: i64) -> Result<String, TerlanPolarsError> {
    array_statistic(input, ddof, false)
}

/// Builds a Polars expression descriptor that applies the array var operation.
pub fn expr_array_var(input: &str, ddof: i64) -> Result<String, TerlanPolarsError> {
    array_statistic(input, ddof, true)
}

/// Builds a Polars expression descriptor that applies the array median operation.
pub fn expr_array_median(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArrayMedian)
}

/// Builds a Polars expression descriptor that applies the array arg min operation.
pub fn expr_array_arg_min(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArrayArgMin)
}

/// Builds a Polars expression descriptor that applies the array arg max operation.
pub fn expr_array_arg_max(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArrayArgMax)
}

/// Builds a Polars expression descriptor that applies the array join operation.
pub fn expr_array_join(
    input: &str,
    separator: &str,
    ignore_nulls: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ArrayJoin {
        input: Box::new(decode(input)?),
        separator: Box::new(decode(separator)?),
        ignore_nulls,
    })
}

/// Builds a Polars expression descriptor that applies the array count matches operation.
pub fn expr_array_count_matches(input: &str, value: &str) -> Result<String, TerlanPolarsError> {
    binary(input, value, BinaryOp::ArrayCountMatches)
}

/// Builds a Polars expression descriptor that applies the array to struct operation.
pub fn expr_array_to_struct(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::ArrayToStruct)
}

fn array_window(
    input: &str,
    first: &str,
    second: Option<&str>,
    kind: ArrayWindowKind,
    as_array: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ArrayWindow {
        input: Box::new(decode(input)?),
        first: Box::new(decode(first)?),
        second: second.map(decode).transpose()?.map(Box::new),
        kind,
        as_array,
    })
}

/// Builds a Polars expression descriptor that applies the array slice operation.
pub fn expr_array_slice(
    input: &str,
    offset: &str,
    length: &str,
    as_array: bool,
) -> Result<String, TerlanPolarsError> {
    array_window(
        input,
        offset,
        Some(length),
        ArrayWindowKind::Slice,
        as_array,
    )
}

/// Builds a Polars expression descriptor that applies the array head operation.
pub fn expr_array_head(
    input: &str,
    length: &str,
    as_array: bool,
) -> Result<String, TerlanPolarsError> {
    array_window(input, length, None, ArrayWindowKind::Head, as_array)
}

/// Builds a Polars expression descriptor that applies the array tail operation.
pub fn expr_array_tail(
    input: &str,
    length: &str,
    as_array: bool,
) -> Result<String, TerlanPolarsError> {
    array_window(input, length, None, ArrayWindowKind::Tail, as_array)
}

/// Builds a Polars expression descriptor that applies the array shift operation.
pub fn expr_array_shift(input: &str, periods: &str) -> Result<String, TerlanPolarsError> {
    binary(input, periods, BinaryOp::ArrayShift)
}

/// Builds a Polars expression descriptor that applies the array explode operation.
pub fn expr_array_explode(
    input: &str,
    empty_as_null: bool,
    keep_nulls: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ArrayExplode {
        input: Box::new(decode(input)?),
        empty_as_null,
        keep_nulls,
    })
}

/// Builds a Polars expression descriptor that applies the array eval operation.
pub fn expr_array_eval(
    input: &str,
    evaluation: &str,
    as_list: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ArrayEval {
        input: Box::new(decode(input)?),
        evaluation: Box::new(decode(evaluation)?),
        as_list,
        aggregate: false,
    })
}

/// Builds a Polars expression descriptor that applies the array agg operation.
pub fn expr_array_agg(input: &str, evaluation: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ArrayEval {
        input: Box::new(decode(input)?),
        evaluation: Box::new(decode(evaluation)?),
        as_list: false,
        aggregate: true,
    })
}

/// Builds a Polars expression descriptor that applies the array sort with operation.
pub fn expr_array_sort_with(
    input: &str,
    descending: bool,
    nulls_last: bool,
    maintain_order: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ArraySortOptions {
        input: Box::new(decode(input)?),
        descending,
        nulls_last,
        maintain_order,
    })
}

fn array_binary_options(
    input: &str,
    value: &str,
    kind: ArrayBinaryOptionsKind,
    option: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ArrayBinaryOptions {
        input: Box::new(decode(input)?),
        value: Box::new(decode(value)?),
        kind,
        option,
    })
}

/// Builds a Polars expression descriptor that applies the array get with operation.
pub fn expr_array_get_with(
    input: &str,
    index: &str,
    null_on_oob: bool,
) -> Result<String, TerlanPolarsError> {
    array_binary_options(input, index, ArrayBinaryOptionsKind::Get, null_on_oob)
}

/// Builds a Polars expression descriptor that applies the array contains with operation.
pub fn expr_array_contains_with(
    input: &str,
    value: &str,
    nulls_equal: bool,
) -> Result<String, TerlanPolarsError> {
    array_binary_options(input, value, ArrayBinaryOptionsKind::Contains, nulls_equal)
}

/// Builds a Polars expression descriptor that applies the categorical categories operation.
pub fn expr_categorical_categories(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::CategoricalCategories)
}

/// Builds a Polars expression descriptor that applies the categorical len bytes operation.
pub fn expr_categorical_len_bytes(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::CategoricalLenBytes)
}

/// Builds a Polars expression descriptor that applies the categorical len chars operation.
pub fn expr_categorical_len_chars(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::CategoricalLenChars)
}

fn categorical_affix(input: &str, value: &str, prefix: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::CategoricalAffix {
        input: Box::new(decode(input)?),
        value: value.into(),
        prefix,
    })
}

/// Builds a Polars expression descriptor that applies the categorical starts with operation.
pub fn expr_categorical_starts_with(
    input: &str,
    prefix: &str,
) -> Result<String, TerlanPolarsError> {
    categorical_affix(input, prefix, true)
}

/// Builds a Polars expression descriptor that applies the categorical ends with operation.
pub fn expr_categorical_ends_with(input: &str, suffix: &str) -> Result<String, TerlanPolarsError> {
    categorical_affix(input, suffix, false)
}

fn categorical_slice(
    input: &str,
    offset: i64,
    length: Option<i64>,
) -> Result<String, TerlanPolarsError> {
    let length = length
        .map(|value| {
            usize::try_from(value).map_err(|_| {
                TerlanPolarsError::new(
                    "invalid_expression",
                    "categorical slice length cannot be negative",
                )
            })
        })
        .transpose()?;
    encode(&ExprNode::CategoricalSlice {
        input: Box::new(decode(input)?),
        offset,
        length,
    })
}

/// Builds a Polars expression descriptor that applies the categorical slice operation.
pub fn expr_categorical_slice(
    input: &str,
    offset: i64,
    length: i64,
) -> Result<String, TerlanPolarsError> {
    categorical_slice(input, offset, Some(length))
}

/// Builds a Polars expression descriptor that applies the categorical slice to end operation.
pub fn expr_categorical_slice_to_end(
    input: &str,
    offset: i64,
) -> Result<String, TerlanPolarsError> {
    categorical_slice(input, offset, None)
}

/// Builds a Polars expression descriptor that applies the binary contains operation.
pub fn expr_binary_contains(input: &str, pattern: &str) -> Result<String, TerlanPolarsError> {
    binary(input, pattern, BinaryOp::BinaryContains)
}

/// Builds a Polars expression descriptor that applies the binary starts with operation.
pub fn expr_binary_starts_with(input: &str, prefix: &str) -> Result<String, TerlanPolarsError> {
    binary(input, prefix, BinaryOp::BinaryStartsWith)
}

/// Builds a Polars expression descriptor that applies the binary ends with operation.
pub fn expr_binary_ends_with(input: &str, suffix: &str) -> Result<String, TerlanPolarsError> {
    binary(input, suffix, BinaryOp::BinaryEndsWith)
}

/// Builds a Polars expression descriptor that applies the binary size bytes operation.
pub fn expr_binary_size_bytes(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BinarySizeBytes)
}

/// Builds a Polars expression descriptor that applies the binary get operation.
pub fn expr_binary_get(input: &str, index: &str) -> Result<String, TerlanPolarsError> {
    binary(input, index, BinaryOp::BinaryGet)
}

/// Builds a Polars expression descriptor that applies the binary head operation.
pub fn expr_binary_head(input: &str, length: &str) -> Result<String, TerlanPolarsError> {
    binary(input, length, BinaryOp::BinaryHead)
}

/// Builds a Polars expression descriptor that applies the binary tail operation.
pub fn expr_binary_tail(input: &str, length: &str) -> Result<String, TerlanPolarsError> {
    binary(input, length, BinaryOp::BinaryTail)
}

/// Builds a Polars expression descriptor that applies the binary slice operation.
pub fn expr_binary_slice(
    input: &str,
    offset: &str,
    length: &str,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::BinarySlice {
        input: Box::new(decode(input)?),
        offset: Box::new(decode(offset)?),
        length: Box::new(decode(length)?),
    })
}

fn binary_decode(
    input: &str,
    encoding: BinaryEncoding,
    strict: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::BinaryDecode {
        input: Box::new(decode(input)?),
        encoding,
        strict,
    })
}

/// Builds a Polars expression descriptor that applies the binary hex decode operation.
pub fn expr_binary_hex_decode(input: &str, strict: bool) -> Result<String, TerlanPolarsError> {
    binary_decode(input, BinaryEncoding::Hex, strict)
}

/// Builds a Polars expression descriptor that applies the binary hex encode operation.
pub fn expr_binary_hex_encode(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BinaryHexEncode)
}

/// Builds a Polars expression descriptor that applies the binary base64 decode operation.
pub fn expr_binary_base64_decode(input: &str, strict: bool) -> Result<String, TerlanPolarsError> {
    binary_decode(input, BinaryEncoding::Base64, strict)
}

/// Builds a Polars expression descriptor that applies the binary base64 encode operation.
pub fn expr_binary_base64_encode(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::BinaryBase64Encode)
}

/// Builds a Polars expression descriptor that applies the binary reinterpret operation.
pub fn expr_binary_reinterpret(
    input: &str,
    data_type: &str,
    is_little_endian: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::BinaryReinterpret {
        input: Box::new(decode(input)?),
        data_type: data_type.into(),
        is_little_endian,
    })
}

/// Builds a Polars expression descriptor that applies the filter expression operation.
pub fn expr_filter_expression(input: &str, predicate: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::FilterExpression {
        input: Box::new(decode(input)?),
        predicate: Box::new(decode(predicate)?),
    })
}

/// Builds a Polars expression descriptor that applies the slice expression operation.
pub fn expr_slice_expression(
    input: &str,
    offset: &str,
    length: &str,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::SliceExpression {
        input: Box::new(decode(input)?),
        offset: Box::new(decode(offset)?),
        length: Box::new(decode(length)?),
    })
}

/// Builds a Polars expression descriptor that applies the append operation.
pub fn expr_append(input: &str, other: &str, upcast: bool) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Append {
        input: Box::new(decode(input)?),
        other: Box::new(decode(other)?),
        upcast,
    })
}

/// Builds a Polars expression descriptor that applies the rechunk operation.
pub fn expr_rechunk(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Rechunk)
}

/// Builds a Polars expression descriptor that applies the arg sort operation.
pub fn expr_arg_sort(
    input: &str,
    descending: bool,
    nulls_last: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ArgSort {
        input: Box::new(decode(input)?),
        descending,
        nulls_last,
    })
}

/// Builds a Polars expression descriptor that applies the index of operation.
pub fn expr_index_of(input: &str, element: &str) -> Result<String, TerlanPolarsError> {
    binary(input, element, BinaryOp::IndexOf)
}

/// Builds a Polars expression descriptor that applies the search sorted operation.
pub fn expr_search_sorted(
    input: &str,
    element: &str,
    side: &str,
    descending: bool,
) -> Result<String, TerlanPolarsError> {
    let side = match side {
        "any" => SearchSide::Any,
        "left" => SearchSide::Left,
        "right" => SearchSide::Right,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "search side must be any, left, or right",
            ))
        }
    };
    encode(&ExprNode::SearchSorted {
        input: Box::new(decode(input)?),
        element: Box::new(decode(element)?),
        side,
        descending,
    })
}

fn gather_expression(
    input: &str,
    index: &str,
    scalar: bool,
    null_on_oob: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::Gather {
        input: Box::new(decode(input)?),
        index: Box::new(decode(index)?),
        scalar,
        null_on_oob,
    })
}

/// Builds a Polars expression descriptor that applies the gather operation.
pub fn expr_gather(
    input: &str,
    index: &str,
    null_on_oob: bool,
) -> Result<String, TerlanPolarsError> {
    gather_expression(input, index, false, null_on_oob)
}

/// Builds a Polars expression descriptor that applies the get expression operation.
pub fn expr_get_expression(
    input: &str,
    index: &str,
    null_on_oob: bool,
) -> Result<String, TerlanPolarsError> {
    gather_expression(input, index, true, null_on_oob)
}

/// Builds a Polars expression descriptor that applies the sort with operation.
pub fn expr_sort_with(
    input: &str,
    descending: bool,
    nulls_last: bool,
) -> Result<String, TerlanPolarsError> {
    expr_sort_with_stability(input, descending, nulls_last, false)
}

/// Sorts an expression with explicit direction, null placement, and stability.
pub fn expr_sort_with_stability(
    input: &str,
    descending: bool,
    nulls_last: bool,
    maintain_order: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::SortOptions {
        input: Box::new(decode(input)?),
        descending,
        nulls_last,
        maintain_order,
    })
}

/// Builds a Polars expression descriptor that applies the reverse operation.
pub fn expr_reverse(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::Reverse)
}

/// Builds a Polars expression descriptor that applies the shift operation.
pub fn expr_shift(input: &str, periods: &str) -> Result<String, TerlanPolarsError> {
    binary(input, periods, BinaryOp::Shift)
}

/// Builds a Polars expression descriptor that applies the shift and fill operation.
pub fn expr_shift_and_fill(
    input: &str,
    periods: &str,
    fill: &str,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ShiftAndFill {
        input: Box::new(decode(input)?),
        periods: Box::new(decode(periods)?),
        fill: Box::new(decode(fill)?),
    })
}

/// Builds a Polars expression descriptor that applies the repeat by operation.
pub fn expr_repeat_by(input: &str, count: &str) -> Result<String, TerlanPolarsError> {
    binary(input, count, BinaryOp::RepeatBy)
}

/// Builds a Polars expression descriptor that applies the diff operation.
pub fn expr_diff(input: &str, periods: &str, behavior: &str) -> Result<String, TerlanPolarsError> {
    let behavior = match behavior {
        "ignore" => DiffBehavior::Ignore,
        "drop" => DiffBehavior::Drop,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "diff null behavior must be ignore or drop",
            ))
        }
    };
    encode(&ExprNode::Diff {
        input: Box::new(decode(input)?),
        periods: Box::new(decode(periods)?),
        behavior,
    })
}

/// Builds a Polars expression descriptor that applies the pct change operation.
pub fn expr_pct_change(input: &str, periods: &str) -> Result<String, TerlanPolarsError> {
    binary(input, periods, BinaryOp::PctChange)
}

/// Builds a Polars expression descriptor that applies the gather every operation.
pub fn expr_gather_every(input: &str, step: i64, offset: i64) -> Result<String, TerlanPolarsError> {
    let step = usize::try_from(step).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_expression",
            "gather step must be greater than zero",
        )
    })?;
    let offset = usize::try_from(offset).map_err(|_| {
        TerlanPolarsError::new("invalid_expression", "gather offset cannot be negative")
    })?;
    if step == 0 {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "gather step must be greater than zero",
        ));
    }
    encode(&ExprNode::GatherEvery {
        input: Box::new(decode(input)?),
        step,
        offset,
    })
}

/// Builds a Polars expression descriptor that applies the extend constant operation.
pub fn expr_extend_constant(
    input: &str,
    value: &str,
    count: &str,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::ExtendConstant {
        input: Box::new(decode(input)?),
        value: Box::new(decode(value)?),
        count: Box::new(decode(count)?),
    })
}

fn sort_by_expression(
    input: &str,
    by: &[String],
    descending: bool,
) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::SortBy {
        input: Box::new(decode(input)?),
        by: decode_many(by)?,
        descending,
    })
}

/// Builds a Polars expression descriptor that applies the sort by ascending operation.
pub fn expr_sort_by_ascending(input: &str, by: &[String]) -> Result<String, TerlanPolarsError> {
    sort_by_expression(input, by, false)
}

/// Builds a Polars expression descriptor that applies the sort by descending operation.
pub fn expr_sort_by_descending(input: &str, by: &[String]) -> Result<String, TerlanPolarsError> {
    sort_by_expression(input, by, true)
}

/// Sorts expression values by one or more keys with complete ordering options.
pub fn expr_sort_by(
    input: &str,
    by: &[String],
    descending: &[bool],
    nulls_last: &[bool],
    maintain_order: bool,
) -> Result<String, TerlanPolarsError> {
    if by.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "sort_by requires at least one ordering expression",
        ));
    }
    for (name, values) in [("descending", descending), ("nulls_last", nulls_last)] {
        if values.len() != 1 && values.len() != by.len() {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                format!("sort_by {name} must contain one value or one value per key"),
            ));
        }
    }
    encode(&ExprNode::SortByOptions {
        input: Box::new(decode(input)?),
        by: decode_many(by)?,
        descending: descending.to_vec(),
        nulls_last: nulls_last.to_vec(),
        maintain_order,
    })
}

fn over_expression(
    input: &str,
    keys: &[String],
    mapping: WindowMappingKind,
) -> Result<String, TerlanPolarsError> {
    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "window expression requires at least one partition key",
        ));
    }
    encode(&ExprNode::Over {
        input: Box::new(decode(input)?),
        keys: decode_many(keys)?,
        mapping,
    })
}

/// Builds a Polars expression descriptor that applies the over operation.
pub fn expr_over(input: &str, keys: &[String]) -> Result<String, TerlanPolarsError> {
    over_expression(input, keys, WindowMappingKind::GroupsToRows)
}

/// Builds a Polars expression descriptor that applies the over explode operation.
pub fn expr_over_explode(input: &str, keys: &[String]) -> Result<String, TerlanPolarsError> {
    over_expression(input, keys, WindowMappingKind::Explode)
}

/// Builds a Polars expression descriptor that applies the over join operation.
pub fn expr_over_join(input: &str, keys: &[String]) -> Result<String, TerlanPolarsError> {
    over_expression(input, keys, WindowMappingKind::Join)
}

/// Builds a Polars expression descriptor that applies the over ordered operation.
pub fn expr_over_ordered(
    input: &str,
    partition_by: &[String],
    order_by: &[String],
    descending: bool,
    nulls_last: bool,
    mapping: &str,
) -> Result<String, TerlanPolarsError> {
    if order_by.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "ordered window expression requires at least one ordering expression",
        ));
    }
    let mapping = match mapping {
        "groups_to_rows" => WindowMappingKind::GroupsToRows,
        "explode" => WindowMappingKind::Explode,
        "join" => WindowMappingKind::Join,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                format!("unsupported window mapping `{mapping}`"),
            ));
        }
    };
    encode(&ExprNode::OverOrdered {
        input: Box::new(decode(input)?),
        partition_by: decode_many(partition_by)?,
        order_by: decode_many(order_by)?,
        descending,
        nulls_last,
        mapping,
    })
}

/// Applies complete partition, ordering, and mapping window options.
pub fn expr_over_with_options(
    input: &str,
    partition_by: &[String],
    order_by: &[String],
    descending: bool,
    nulls_last: bool,
    mapping: &str,
) -> Result<String, TerlanPolarsError> {
    expr_over_ordered(
        input,
        partition_by,
        order_by,
        descending,
        nulls_last,
        mapping,
    )
}

/// Builds a Polars expression descriptor that applies the head expression operation.
pub fn expr_head_expression(input: &str, limit: i64) -> Result<String, TerlanPolarsError> {
    let limit = usize::try_from(limit).map_err(|_| {
        TerlanPolarsError::new("invalid_expression", "head limit cannot be negative")
    })?;
    encode(&ExprNode::HeadExpression {
        input: Box::new(decode(input)?),
        limit,
    })
}

/// Builds a Polars expression descriptor that applies the tail expression operation.
pub fn expr_tail_expression(input: &str, limit: i64) -> Result<String, TerlanPolarsError> {
    let limit = usize::try_from(limit).map_err(|_| {
        TerlanPolarsError::new("invalid_expression", "tail limit cannot be negative")
    })?;
    encode(&ExprNode::TailExpression {
        input: Box::new(decode(input)?),
        limit,
    })
}

/// Builds a Polars expression descriptor that applies the as struct operation.
pub fn expr_as_struct(inputs: &[String]) -> Result<String, TerlanPolarsError> {
    if inputs.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "as_struct requires at least one field",
        ));
    }
    encode(&ExprNode::Struct {
        inputs: decode_many(inputs)?,
    })
}

/// Builds a Polars expression descriptor that applies the struct field operation.
pub fn expr_struct_field(input: &str, name: &str) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StructField {
        input: Box::new(decode(input)?),
        name: name.into(),
    })
}

/// Builds a Polars expression descriptor that applies the struct field at operation.
pub fn expr_struct_field_at(input: &str, index: i64) -> Result<String, TerlanPolarsError> {
    encode(&ExprNode::StructFieldAt {
        input: Box::new(decode(input)?),
        index,
    })
}

/// Builds a Polars expression descriptor that applies the struct fields operation.
pub fn expr_struct_fields(input: &str, names: &[String]) -> Result<String, TerlanPolarsError> {
    if names.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "struct_fields requires at least one field name",
        ));
    }
    encode(&ExprNode::StructFields {
        input: Box::new(decode(input)?),
        names: names.to_vec(),
    })
}

/// Builds a Polars expression descriptor that applies the struct rename fields operation.
pub fn expr_struct_rename_fields(
    input: &str,
    names: &[String],
) -> Result<String, TerlanPolarsError> {
    if names.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "struct_rename_fields requires at least one field name",
        ));
    }
    encode(&ExprNode::StructRenameFields {
        input: Box::new(decode(input)?),
        names: names.to_vec(),
    })
}

/// Builds a Polars expression descriptor that applies the struct json encode operation.
pub fn expr_struct_json_encode(input: &str) -> Result<String, TerlanPolarsError> {
    unary(input, UnaryOp::StructJsonEncode)
}

/// Builds a Polars expression descriptor that applies the struct with fields operation.
pub fn expr_struct_with_fields(
    input: &str,
    fields: &[String],
) -> Result<String, TerlanPolarsError> {
    if fields.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "struct_with_fields requires at least one field expression",
        ));
    }
    encode(&ExprNode::StructWithFields {
        input: Box::new(decode(input)?),
        fields: decode_many(fields)?,
    })
}

#[cfg(not(feature = "real-polars"))]
/// Resolving an expression field requires the real Polars schema engine.
pub fn expr_to_field(
    _input: &str,
    _names: &[String],
    _data_types: &[String],
) -> Result<String, TerlanPolarsError> {
    Err(crate::unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Resolves an expression's output field against a portable input schema.
pub fn expr_to_field(
    input: &str,
    names: &[String],
    data_types: &[String],
) -> Result<String, TerlanPolarsError> {
    if names.len() != data_types.len() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "field schema names and data types must have equal lengths",
        ));
    }
    let schema = polars::prelude::Schema::from_iter_check_duplicates(
        names
            .iter()
            .zip(data_types)
            .map(|(name, data_type)| Ok((name.as_str().into(), parse_data_type(data_type)?)))
            .collect::<Result<Vec<_>, TerlanPolarsError>>()?,
    )?;
    let field = compile_one(input)?.to_field(&schema)?;
    let data_type = crate::data_types::encode_data_type(field.dtype())?;
    data_type_field(field.name().as_str(), &data_type)
}

#[cfg(not(feature = "real-polars"))]
fn metadata_unavailable<T>() -> Result<T, TerlanPolarsError> {
    Err(crate::unavailable_error())
}

#[cfg(not(feature = "real-polars"))]
pub fn expr_meta_root_names(_input: &str) -> Result<Vec<String>, TerlanPolarsError> {
    metadata_unavailable()
}

#[cfg(feature = "real-polars")]
/// Builds a Polars expression descriptor that applies the meta root names operation.
pub fn expr_meta_root_names(input: &str) -> Result<Vec<String>, TerlanPolarsError> {
    Ok(compile_one(input)?
        .meta()
        .root_names()
        .into_iter()
        .map(|name| name.to_string())
        .collect())
}

#[cfg(not(feature = "real-polars"))]
pub fn expr_meta_output_name(_input: &str) -> Result<String, TerlanPolarsError> {
    metadata_unavailable()
}

#[cfg(feature = "real-polars")]
/// Builds a Polars expression descriptor that applies the meta output name operation.
pub fn expr_meta_output_name(input: &str) -> Result<String, TerlanPolarsError> {
    Ok(compile_one(input)?.meta().output_name()?.to_string())
}

#[cfg(not(feature = "real-polars"))]
pub fn expr_meta_has_multiple_outputs(_input: &str) -> Result<bool, TerlanPolarsError> {
    metadata_unavailable()
}

#[cfg(feature = "real-polars")]
/// Builds a Polars expression descriptor that applies the meta has multiple outputs operation.
pub fn expr_meta_has_multiple_outputs(input: &str) -> Result<bool, TerlanPolarsError> {
    Ok(compile_one(input)?.meta().has_multiple_outputs())
}

#[cfg(not(feature = "real-polars"))]
pub fn expr_meta_is_column(_input: &str) -> Result<bool, TerlanPolarsError> {
    metadata_unavailable()
}

#[cfg(feature = "real-polars")]
/// Builds a Polars expression descriptor that applies the meta is column operation.
pub fn expr_meta_is_column(input: &str) -> Result<bool, TerlanPolarsError> {
    Ok(compile_one(input)?.meta().is_column())
}

#[cfg(not(feature = "real-polars"))]
pub fn expr_meta_is_simple_projection(_input: &str) -> Result<bool, TerlanPolarsError> {
    metadata_unavailable()
}

#[cfg(feature = "real-polars")]
/// Builds a Polars expression descriptor that applies the meta is simple projection operation.
pub fn expr_meta_is_simple_projection(input: &str) -> Result<bool, TerlanPolarsError> {
    Ok(compile_one(input)?.meta().is_simple_projection(None))
}

#[cfg(not(feature = "real-polars"))]
pub fn expr_meta_is_column_selection(
    _input: &str,
    _allow_aliasing: bool,
) -> Result<bool, TerlanPolarsError> {
    metadata_unavailable()
}

#[cfg(feature = "real-polars")]
/// Builds a Polars expression descriptor that applies the meta is column selection operation.
pub fn expr_meta_is_column_selection(
    input: &str,
    allow_aliasing: bool,
) -> Result<bool, TerlanPolarsError> {
    Ok(compile_one(input)?
        .meta()
        .is_column_selection(allow_aliasing))
}

#[cfg(not(feature = "real-polars"))]
pub fn expr_meta_is_literal(
    _input: &str,
    _allow_aliasing: bool,
) -> Result<bool, TerlanPolarsError> {
    metadata_unavailable()
}

#[cfg(feature = "real-polars")]
/// Builds a Polars expression descriptor that applies the meta is literal operation.
pub fn expr_meta_is_literal(input: &str, allow_aliasing: bool) -> Result<bool, TerlanPolarsError> {
    Ok(compile_one(input)?.meta().is_literal(allow_aliasing))
}

#[cfg(not(feature = "real-polars"))]
pub fn expr_meta_is_regex_projection(_input: &str) -> Result<bool, TerlanPolarsError> {
    metadata_unavailable()
}

#[cfg(feature = "real-polars")]
/// Builds a Polars expression descriptor that applies the meta is regex projection operation.
pub fn expr_meta_is_regex_projection(input: &str) -> Result<bool, TerlanPolarsError> {
    Ok(compile_one(input)?.meta().is_regex_projection())
}

#[cfg(not(feature = "real-polars"))]
pub fn expr_meta_format_tree(
    _input: &str,
    _display_as_dot: bool,
) -> Result<String, TerlanPolarsError> {
    metadata_unavailable()
}

#[cfg(feature = "real-polars")]
/// Builds a Polars expression descriptor that applies the meta format tree operation.
pub fn expr_meta_format_tree(
    input: &str,
    display_as_dot: bool,
) -> Result<String, TerlanPolarsError> {
    use polars::prelude::{DataType, Schema};

    let expression = compile_one(input)?;
    let schema = Schema::from_iter(
        expression
            .clone()
            .meta()
            .root_names()
            .into_iter()
            .map(|name| (name, DataType::Int64)),
    );
    let formatted = expression
        .meta()
        .into_tree_formatter(display_as_dot, Some(&schema))?
        .to_string();
    Ok(formatted)
}

#[cfg(feature = "real-polars")]
pub(crate) fn parse_data_type(value: &str) -> Result<polars::prelude::DataType, TerlanPolarsError> {
    use polars::prelude::{Categories, DataType, Field, FrozenCategories, TimeUnit, TimeZone};

    if value.starts_with('{') {
        let descriptor = serde_json::from_str::<DataTypeDescriptor>(value).map_err(|error| {
            TerlanPolarsError::new(
                "invalid_data_type",
                format!("invalid data type descriptor: {error}"),
            )
        })?;
        return match descriptor {
            DataTypeDescriptor::List { element } => {
                Ok(DataType::List(Box::new(parse_data_type(&element)?)))
            }
            DataTypeDescriptor::Map { key, value } => {
                let data_type = DataType::Map(
                    Box::new(parse_data_type(&key)?),
                    Box::new(parse_data_type(&value)?),
                );
                data_type.ensure_valid_map_dtype()?;
                Ok(data_type)
            }
            DataTypeDescriptor::Array { element, width } => {
                Ok(DataType::Array(Box::new(parse_data_type(&element)?), width))
            }
            DataTypeDescriptor::Decimal { precision, scale } => {
                if !(1..=38).contains(&precision) || scale > precision {
                    return Err(TerlanPolarsError::new(
                        "invalid_data_type",
                        "invalid decimal precision or scale",
                    ));
                }
                Ok(DataType::Decimal(precision, scale))
            }
            DataTypeDescriptor::Categorical => Ok(DataType::from_categories(Categories::global())),
            DataTypeDescriptor::Enum { categories } => {
                let categories = FrozenCategories::new(categories.iter().map(String::as_str))
                    .map_err(|error| {
                        TerlanPolarsError::new(
                            "invalid_data_type",
                            format!("invalid enum categories: {error}"),
                        )
                    })?;
                Ok(DataType::from_frozen_categories(categories))
            }
            DataTypeDescriptor::Datetime {
                time_unit,
                time_zone,
            } => {
                let time_unit = match time_unit.as_str() {
                    "milliseconds" => TimeUnit::Milliseconds,
                    "microseconds" => TimeUnit::Microseconds,
                    "nanoseconds" => TimeUnit::Nanoseconds,
                    _ => {
                        return Err(TerlanPolarsError::new(
                            "invalid_data_type",
                            format!("unsupported datetime time unit `{time_unit}`"),
                        ));
                    }
                };
                let time_zone = TimeZone::opt_try_new(Some(time_zone.as_str()))
                    .map_err(|error| {
                        TerlanPolarsError::new(
                            "invalid_data_type",
                            format!("invalid datetime time zone: {error}"),
                        )
                    })?
                    .ok_or_else(|| {
                        TerlanPolarsError::new(
                            "invalid_data_type",
                            "timezone-aware datetime requires a non-empty time zone",
                        )
                    })?;
                Ok(DataType::Datetime(time_unit, Some(time_zone)))
            }
            DataTypeDescriptor::Struct { fields } => {
                let fields = fields
                    .iter()
                    .map(|field| {
                        let descriptor = serde_json::from_str::<DataTypeDescriptor>(field)
                            .map_err(|error| {
                                TerlanPolarsError::new(
                                    "invalid_data_type",
                                    format!("invalid struct field descriptor: {error}"),
                                )
                            })?;
                        let DataTypeDescriptor::Field { name, data_type } = descriptor else {
                            return Err(TerlanPolarsError::new(
                                "invalid_data_type",
                                "struct type contains a non-field descriptor",
                            ));
                        };
                        Ok(Field::new(name.into(), parse_data_type(&data_type)?))
                    })
                    .collect::<Result<Vec<_>, TerlanPolarsError>>()?;
                Ok(DataType::Struct(fields))
            }
            DataTypeDescriptor::Extension {
                name,
                metadata,
                storage,
            } => {
                let storage = parse_data_type(&storage)?;
                let extension = polars::datatypes::extension::get_extension_type_or_generic(
                    &name,
                    &storage,
                    metadata.as_deref(),
                );
                Ok(DataType::Extension(extension, Box::new(storage)))
            }
            DataTypeDescriptor::Field { .. } => Err(TerlanPolarsError::new(
                "invalid_data_type",
                "a field descriptor is not a standalone data type",
            )),
        };
    }

    // Terlan compiler versions that split letter-to-digit atom boundaries
    // encode `Int32` as `int_32`; older versions encode it as `int32`.
    // Normalize only underscores immediately before digits so both boundary
    // spellings remain compatible without changing meaningful word separators.
    let mut normalized = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '_' && chars.peek().is_some_and(char::is_ascii_digit) {
            continue;
        }
        normalized.push(ch);
    }
    let value = normalized.as_str();

    match value {
        "Boolean" | "Bool" | "boolean" => Ok(DataType::Boolean),
        "Int8" | "int8" => Ok(DataType::Int8),
        "Int16" | "int16" => Ok(DataType::Int16),
        "Int32" | "int32" => Ok(DataType::Int32),
        "Int64" | "int64" => Ok(DataType::Int64),
        "Int128" | "int128" => Ok(DataType::Int128),
        "UInt8" | "uint8" | "u_int8" => Ok(DataType::UInt8),
        "UInt16" | "uint16" | "u_int16" => Ok(DataType::UInt16),
        "UInt32" | "uint32" | "u_int32" => Ok(DataType::UInt32),
        "UInt64" | "uint64" | "u_int64" => Ok(DataType::UInt64),
        "UInt128" | "uint128" | "u_int128" => Ok(DataType::UInt128),
        "Float16" | "float16" => Ok(DataType::Float16),
        "Float32" | "float32" => Ok(DataType::Float32),
        "Float64" | "float64" => Ok(DataType::Float64),
        "String" | "Utf8" | "string" | "utf8" => Ok(DataType::String),
        "Binary" | "BinaryType" | "binary" | "binary_type" => Ok(DataType::Binary),
        "BinaryOffset" | "BinaryOffsetType" | "binary_offset" | "binary_offset_type" => {
            Ok(DataType::BinaryOffset)
        }
        "Date" | "DateType" | "date_type" => Ok(DataType::Date),
        "Time" | "TimeType" | "time_type" => Ok(DataType::Time),
        "DatetimeMilliseconds" | "datetime_milliseconds" => {
            Ok(DataType::Datetime(TimeUnit::Milliseconds, None))
        }
        "DatetimeMicroseconds" | "datetime_microseconds" => {
            Ok(DataType::Datetime(TimeUnit::Microseconds, None))
        }
        "DatetimeNanoseconds" | "datetime_nanoseconds" => {
            Ok(DataType::Datetime(TimeUnit::Nanoseconds, None))
        }
        "DurationMilliseconds" | "duration_milliseconds" => {
            Ok(DataType::Duration(TimeUnit::Milliseconds))
        }
        "DurationMicroseconds" | "duration_microseconds" => {
            Ok(DataType::Duration(TimeUnit::Microseconds))
        }
        "DurationNanoseconds" | "duration_nanoseconds" => {
            Ok(DataType::Duration(TimeUnit::Nanoseconds))
        }
        "Null" | "NullType" | "null_type" => Ok(DataType::Null),
        _ => Err(TerlanPolarsError::new(
            "invalid_expression",
            format!("unsupported data type `{value}`"),
        )),
    }
}

#[cfg(feature = "real-polars")]
fn titlecase_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first
                    .to_uppercase()
                    .chain(chars.flat_map(char::to_lowercase))
                    .collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(feature = "real-polars")]
fn parse_data_types(
    values: &[String],
) -> Result<Vec<polars::prelude::DataType>, TerlanPolarsError> {
    values.iter().map(|value| parse_data_type(value)).collect()
}

#[cfg(feature = "real-polars")]
/// Returns the Polars 2.0 compatibility error for a removed cast, if any.
pub(crate) fn polars2_cast_error(
    source: &polars::prelude::DataType,
    target: &polars::prelude::DataType,
    strict: bool,
) -> Option<String> {
    use polars::prelude::DataType;

    let source_is_categorical =
        matches!(source, DataType::Categorical(_, _) | DataType::Enum(_, _));
    let target_is_categorical =
        matches!(target, DataType::Categorical(_, _) | DataType::Enum(_, _));

    if source == &DataType::String
        && matches!(
            target,
            DataType::Date | DataType::Datetime(_, _) | DataType::Time
        )
    {
        return Some(format!(
            "casting from String to {target} is not supported in Polars 2.0; use the string temporal namespace"
        ));
    }
    if source.is_integer() && target_is_categorical {
        return Some(format!(
            "casting from {source} to {target} is not supported in Polars 2.0; use categorical conversion"
        ));
    }
    if source_is_categorical && target.is_integer() {
        return Some(format!(
            "casting from {source} to {target} is not supported in Polars 2.0; use the categorical physical representation"
        ));
    }
    if !source.is_nested() && matches!(target, DataType::List(_)) {
        return Some(format!(
            "casting from {source} to a List is not supported in Polars 2.0; use an explicit unit-list operation"
        ));
    }
    if strict {
        if let (DataType::Struct(source_fields), DataType::Struct(target_fields)) = (source, target)
        {
            let same_shape = source_fields.len() == target_fields.len()
                && target_fields.iter().all(|target| {
                    source_fields
                        .iter()
                        .any(|source| source.name == target.name)
                });
            if !same_shape {
                return Some(format!(
                    "strict cast from {source} to {target} requires the same number of struct fields and matching field names"
                ));
            }
        }
    }
    None
}

#[cfg(feature = "real-polars")]
fn invalid_operation(message: String) -> polars::error::PolarsError {
    polars::error::PolarsError::InvalidOperation(message.into())
}

#[cfg(feature = "real-polars")]
fn polars2_cast_expr(
    input: polars::prelude::Expr,
    target: polars::prelude::DataType,
    options: polars::chunked_array::cast::CastOptions,
    strict: bool,
) -> polars::prelude::Expr {
    use polars::prelude::{Field, IntoColumn};

    let execute_target = target.clone();
    let schema_target = target;
    input.map(
        move |column| {
            let series = column.as_materialized_series();
            if let Some(message) = polars2_cast_error(series.dtype(), &execute_target, strict) {
                return Err(invalid_operation(message));
            }
            Ok(series
                .cast_with_options(&execute_target, options)?
                .into_column())
        },
        move |_schema, field| {
            if let Some(message) = polars2_cast_error(field.dtype(), &schema_target, strict) {
                return Err(invalid_operation(message));
            }
            Ok(Field::new(field.name().clone(), schema_target.clone()))
        },
    )
}

#[cfg(feature = "real-polars")]
fn validate_membership_types(
    needle: &polars::prelude::DataType,
    values: &polars::prelude::DataType,
) -> polars::prelude::PolarsResult<()> {
    use polars::prelude::DataType;

    let value = match values {
        DataType::List(inner) | DataType::Array(inner, _) => inner.as_ref(),
        _ => values,
    };
    if needle == value || needle == &DataType::Null || value == &DataType::Null {
        return Ok(());
    }
    if needle.is_primitive_numeric() && value.is_primitive_numeric() {
        let both_integers = needle.is_integer()
            && value.is_integer()
            && !matches!(
                (needle, value),
                (DataType::UInt128, signed) | (signed, DataType::UInt128)
                    if signed.is_signed_integer()
            );
        let both_floats = needle.is_float() && value.is_float();
        if both_integers || both_floats {
            return Ok(());
        }
        return Err(invalid_operation(format!(
            "membership cannot losslessly compare {needle} with {values} in Polars 2.0; cast one operand explicitly"
        )));
    }
    if matches!(needle, DataType::Decimal(_, _)) && value.is_float()
        || needle.is_float() && matches!(value, DataType::Decimal(_, _))
    {
        return Err(invalid_operation(format!(
            "membership cannot compare Decimal and floating-point data in Polars 2.0 ({needle} and {values})"
        )));
    }
    if let (DataType::Datetime(_, needle_zone), DataType::Datetime(_, value_zone)) = (needle, value)
    {
        if needle_zone.is_some() != value_zone.is_some() {
            return Err(invalid_operation(format!(
                "membership cannot compare naive and time-zone-aware datetimes ({needle} and {values})"
            )));
        }
    }
    Ok(())
}

#[cfg(feature = "real-polars")]
fn validated_membership_values(
    values: polars::prelude::Expr,
    needle: polars::prelude::Expr,
) -> polars::prelude::Expr {
    values.map_many(
        |columns| {
            let values = columns[0].as_materialized_series();
            let needle = columns[1].as_materialized_series();
            validate_membership_types(needle.dtype(), values.dtype())?;
            Ok(columns[0].clone())
        },
        &[needle],
        |_schema, fields| {
            validate_membership_types(fields[1].dtype(), fields[0].dtype())?;
            Ok(fields[0].clone())
        },
    )
}

#[cfg(feature = "real-polars")]
fn require_numeric(input: polars::prelude::Expr, operation: &'static str) -> polars::prelude::Expr {
    input.map(
        move |column| {
            if !column.dtype().is_numeric() {
                return Err(invalid_operation(format!(
                    "{operation} is not supported for dtype {} in Polars 2.0",
                    column.dtype()
                )));
            }
            Ok(column)
        },
        move |_schema, field| {
            if !field.dtype().is_numeric() {
                return Err(invalid_operation(format!(
                    "{operation} is not supported for dtype {} in Polars 2.0",
                    field.dtype()
                )));
            }
            Ok(field.clone())
        },
    )
}

#[cfg(feature = "real-polars")]
fn reject_boolean_integer_pair(
    left: polars::prelude::Expr,
    right: polars::prelude::Expr,
    operation: &'static str,
) -> polars::prelude::Expr {
    let validate = move |left: &polars::prelude::DataType,
                         right: &polars::prelude::DataType|
          -> polars::prelude::PolarsResult<()> {
        if (left.is_bool() && right.is_integer()) || (right.is_bool() && left.is_integer()) {
            Err(invalid_operation(format!(
                "{operation} between Boolean and integer data is not supported in Polars 2.0"
            )))
        } else {
            Ok(())
        }
    };
    left.map_many(
        move |columns| {
            validate(columns[0].dtype(), columns[1].dtype())?;
            Ok(columns[0].clone())
        },
        &[right],
        move |_schema, fields| {
            validate(fields[0].dtype(), fields[1].dtype())?;
            Ok(fields[0].clone())
        },
    )
}

#[cfg(feature = "real-polars")]
fn reject_duration(input: polars::prelude::Expr, operation: &'static str) -> polars::prelude::Expr {
    input.map(
        move |column| {
            if matches!(column.dtype(), polars::prelude::DataType::Duration(_)) {
                return Err(invalid_operation(format!(
                    "{operation} is not supported for Duration in Polars 2.0"
                )));
            }
            Ok(column)
        },
        move |_schema, field| {
            if matches!(field.dtype(), polars::prelude::DataType::Duration(_)) {
                return Err(invalid_operation(format!(
                    "{operation} is not supported for Duration in Polars 2.0"
                )));
            }
            Ok(field.clone())
        },
    )
}

#[cfg(feature = "real-polars")]
fn require_list(input: polars::prelude::Expr, operation: &'static str) -> polars::prelude::Expr {
    input.map(
        move |column| {
            if !matches!(column.dtype(), polars::prelude::DataType::List(_)) {
                return Err(invalid_operation(format!(
                    "{operation} requires a List input in Polars 2.0; wrap flat values explicitly"
                )));
            }
            Ok(column)
        },
        move |_schema, field| {
            if !matches!(field.dtype(), polars::prelude::DataType::List(_)) {
                return Err(invalid_operation(format!(
                    "{operation} requires a List input in Polars 2.0; wrap flat values explicitly"
                )));
            }
            Ok(field.clone())
        },
    )
}

#[cfg(feature = "real-polars")]
fn require_struct_width(input: polars::prelude::Expr, expected: usize) -> polars::prelude::Expr {
    let validate = move |dtype: &polars::prelude::DataType| -> polars::prelude::PolarsResult<()> {
        if let polars::prelude::DataType::Struct(fields) = dtype {
            if fields.len() != expected {
                return Err(polars::error::PolarsError::SchemaMismatch(
                    format!(
                        "struct.rename_fields requires exactly {} names, received {expected}",
                        fields.len()
                    )
                    .into(),
                ));
            }
        }
        Ok(())
    };
    input.map(
        move |column| {
            validate(column.dtype())?;
            Ok(column)
        },
        move |_schema, field| {
            validate(field.dtype())?;
            Ok(field.clone())
        },
    )
}

#[cfg(feature = "real-polars")]
fn preserve_outer_validity(
    structure: polars::prelude::Expr,
    source: polars::prelude::Expr,
) -> polars::prelude::Expr {
    use polars::prelude::{IntoColumn, IntoSeries};

    structure.map_many(
        |columns| {
            let validity = columns[1].as_materialized_series().rechunk_validity();
            Ok(columns[0]
                .as_materialized_series()
                .struct_()?
                .clone()
                .with_outer_validity(validity)
                .into_series()
                .into_column())
        },
        &[source],
        |_schema, fields| {
            if fields[0].dtype().is_known() {
                return Ok(fields[0].clone());
            }
            if let polars::prelude::DataType::Array(inner, width) = fields[1].dtype() {
                let struct_fields = (0..*width)
                    .map(|index| {
                        polars::prelude::Field::new(
                            format!("field_{index}").into(),
                            inner.as_ref().clone(),
                        )
                    })
                    .collect();
                return Ok(polars::prelude::Field::new(
                    fields[1].name().clone(),
                    polars::prelude::DataType::Struct(struct_fields),
                ));
            }
            Ok(fields[0].clone())
        },
    )
}

#[cfg(feature = "real-polars")]
pub(crate) fn compile_one(value: &str) -> Result<polars::prelude::Expr, TerlanPolarsError> {
    compile(&decode(value)?)
}

#[cfg(feature = "real-polars")]
pub(crate) fn compile_many(
    values: &[String],
) -> Result<Vec<polars::prelude::Expr>, TerlanPolarsError> {
    values
        .iter()
        .map(|value| compile(&decode(value)?))
        .collect()
}

#[cfg(feature = "real-polars")]
fn transformed_name(name: &str, transform: &NameTransformKind) -> String {
    match transform {
        NameTransformKind::Prefix(value) => format!("{value}{name}"),
        NameTransformKind::Suffix(value) => format!("{name}{value}"),
        NameTransformKind::Replace { pattern, value } => name.replace(pattern, value),
        NameTransformKind::Lowercase => name.to_lowercase(),
        NameTransformKind::Uppercase => name.to_uppercase(),
    }
}

#[cfg(feature = "real-polars")]
fn escape_regex_literal(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(
            character,
            '.' | '+' | '*' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\'
        ) {
            output.push('\\');
        }
        output.push(character);
    }
    output
}

#[cfg(feature = "real-polars")]
fn compile_selector(node: &ExprNode) -> Result<polars::prelude::Selector, TerlanPolarsError> {
    use polars::prelude::{DataType, DataTypeSelector, Selector};

    Ok(match node {
        ExprNode::All => Selector::Wildcard,
        ExprNode::Columns { names } => polars::lazy::dsl::cols(names.clone()),
        ExprNode::DTypeColumns { data_types } => {
            polars::lazy::dsl::dtype_cols(parse_data_types(data_types)?).as_selector()
        }
        ExprNode::SelectorLeaf { kind, value } => match kind {
            SelectorLeafKind::All => Selector::Wildcard,
            SelectorLeafKind::Numeric => DataTypeSelector::Numeric.as_selector(),
            SelectorLeafKind::String => {
                polars::lazy::dsl::dtype_col(&DataType::String).as_selector()
            }
            SelectorLeafKind::StartsWith => {
                Selector::Matches(format!("^{}.*$", escape_regex_literal(value)).into())
            }
            SelectorLeafKind::Contains => {
                Selector::Matches(format!("^.*{}.*$", escape_regex_literal(value)).into())
            }
            SelectorLeafKind::First => polars::lazy::dsl::first(),
        },
        ExprNode::SelectorBinary { kind, left, right } => {
            let left = compile_selector(left)?;
            let right = compile_selector(right)?;
            match kind {
                SelectorSetOp::Union => left | right,
                SelectorSetOp::Intersection => left & right,
                SelectorSetOp::Difference => left - right,
                SelectorSetOp::ExclusiveOr => left ^ right,
            }
        }
        ExprNode::SelectorComplement { input } => !compile_selector(input)?,
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_expression",
                "expression is not a selector",
            ));
        }
    })
}

#[cfg(feature = "real-polars")]
fn series_from_ipc_payload(value: &str) -> Result<polars::prelude::Series, TerlanPolarsError> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use polars::prelude::{IpcReader, SerReader};

    let bytes = STANDARD.decode(value).map_err(|error| {
        TerlanPolarsError::new(
            "invalid_expression",
            format!("cannot decode Series expression payload: {error}"),
        )
    })?;
    let frame = IpcReader::new(std::io::Cursor::new(bytes)).finish()?;
    let column = frame.select_at_idx(0).ok_or_else(|| {
        TerlanPolarsError::new(
            "invalid_expression",
            "Series expression payload contains no column",
        )
    })?;
    Ok(column.as_materialized_series().clone())
}

#[cfg(feature = "real-polars")]
fn compile(node: &ExprNode) -> Result<polars::prelude::Expr, TerlanPolarsError> {
    use chrono::NaiveDate;
    use polars::prelude::{
        all, col, cols, dtype_cols, len, lit, when, ClosedInterval, IntoColumn, NamedFrom,
        RoundMode, Series, StrptimeOptions, TimeUnit,
    };

    Ok(match node {
        ExprNode::Parameter { index } => {
            return Err(TerlanPolarsError::new(
                "unresolved_udf_parameter",
                format!("expression UDF parameter {index} was not substituted"),
            ));
        }
        ExprNode::Column { name } => col(name),
        ExprNode::Columns { names } => cols(names.clone()).as_expr(),
        ExprNode::DTypeColumns { data_types } => dtype_cols(parse_data_types(data_types)?)
            .as_selector()
            .as_expr(),
        ExprNode::All => all().as_expr(),
        ExprNode::SelectorLeaf { .. }
        | ExprNode::SelectorBinary { .. }
        | ExprNode::SelectorComplement { .. } => compile_selector(node)?.as_expr(),
        ExprNode::ConcatList { inputs } => {
            let inputs = inputs.iter().map(compile).collect::<Result<Vec<_>, _>>()?;
            polars::lazy::dsl::concat_list(&inputs)?
        }
        ExprNode::IntRange {
            start,
            end,
            step,
            data_type,
        } => {
            let data_type = parse_data_type(data_type)?;
            if !data_type.is_integer() {
                return Err(TerlanPolarsError::new(
                    "invalid_expression",
                    "integer range data type must be an integer type",
                ));
            }
            polars::lazy::dsl::int_range(compile(start)?, compile(end)?, *step, data_type)
        }
        ExprNode::IntRanges {
            start,
            end,
            step,
            data_type,
        } => {
            let data_type = parse_data_type(data_type)?;
            if !data_type.is_integer() {
                return Err(TerlanPolarsError::new(
                    "invalid_expression",
                    "integer ranges data type must be an integer type",
                ));
            }
            polars::lazy::dsl::int_ranges(compile(start)?, compile(end)?, compile(step)?, data_type)
        }
        ExprNode::Literal { value } => match value {
            Literal::Null => lit(polars::prelude::NULL),
            Literal::String(value) => lit(value.clone()),
            Literal::Int(value) => lit(*value),
            Literal::Float(value) => lit(*value),
            Literal::Bool(value) => lit(*value),
        },
        ExprNode::SeriesLiteral { ipc_base64 } => lit(series_from_ipc_payload(ipc_base64)?),
        ExprNode::Date { iso } => {
            let date = NaiveDate::parse_from_str(iso, "%Y-%m-%d").map_err(|error| {
                TerlanPolarsError::new("invalid_date", format!("invalid ISO date `{iso}`: {error}"))
            })?;
            lit(date)
        }
        ExprNode::Sql { query } => polars::sql::sql_expr(query)
            .map_err(|error| TerlanPolarsError::new("sql_error", error.to_string()))?,
        ExprNode::Len => len(),
        ExprNode::Element => polars::lazy::dsl::element(),
        ExprNode::Unary { kind, input } => {
            let input = compile(input)?;
            match kind {
                UnaryOp::Millennium => input.dt().millennium(),
                UnaryOp::Century => input.dt().century(),
                UnaryOp::Year => input.dt().year(),
                UnaryOp::IsLeapYear => input.dt().is_leap_year(),
                UnaryOp::IsoYear => input.dt().iso_year(),
                UnaryOp::Month => input.dt().month(),
                UnaryOp::DaysInMonth => input.dt().days_in_month(),
                UnaryOp::Quarter => input.dt().quarter(),
                UnaryOp::Week => input.dt().week(),
                UnaryOp::Day => input.dt().day(),
                UnaryOp::Weekday => input.dt().weekday(),
                UnaryOp::OrdinalDay => input.dt().ordinal_day(),
                UnaryOp::TimeOfDay => input.dt().time(),
                UnaryOp::CalendarDate => input.dt().date(),
                UnaryOp::LocalDatetime => input.dt().replace_time_zone(
                    None,
                    lit("raise"),
                    polars::prelude::NonExistent::Raise,
                ),
                UnaryOp::Hour => input.dt().hour(),
                UnaryOp::Minute => input.dt().minute(),
                UnaryOp::Second => input.dt().second(),
                UnaryOp::Millisecond => input.dt().millisecond(),
                UnaryOp::Microsecond => input.dt().microsecond(),
                UnaryOp::Nanosecond => input.dt().nanosecond(),
                UnaryOp::MonthStart => input.dt().month_start(),
                UnaryOp::MonthEnd => input.dt().month_end(),
                UnaryOp::Mean => input.mean(),
                UnaryOp::Max => input.max(),
                UnaryOp::Not => input.not(),
                UnaryOp::IsNull => input.is_null(),
                UnaryOp::IsNotNull => input.is_not_null(),
                UnaryOp::IsFinite => input.is_finite(),
                UnaryOp::IsInfinite => input.is_infinite(),
                UnaryOp::IsNan => input.is_nan(),
                UnaryOp::IsNotNan => input.is_not_nan(),
                UnaryOp::IsFirstDistinct => input.is_first_distinct(),
                UnaryOp::IsLastDistinct => input.is_last_distinct(),
                UnaryOp::IsUnique => input.is_unique(),
                UnaryOp::IsDuplicated => input.is_duplicated(),
                UnaryOp::HasNulls => input.has_nulls(),
                UnaryOp::NUnique => input.n_unique(),
                UnaryOp::ApproxNUnique => input.approx_n_unique(),
                UnaryOp::ValueCounts => input.value_counts(false, false, "count", false),
                UnaryOp::UniqueStable => input.unique_stable(),
                UnaryOp::UniqueCounts => input.unique_counts(),
                UnaryOp::StringLenBytes => input.str().len_bytes(),
                UnaryOp::StringLenChars => input.str().len_chars(),
                UnaryOp::StringTitlecase => input.map(
                    |column| {
                        let name = column.name().clone();
                        let values = column.str()?;
                        let result = values
                            .iter()
                            .map(|value| value.map(titlecase_text))
                            .collect::<polars::prelude::StringChunked>()
                            .with_name(name);
                        Ok(result.into_column())
                    },
                    |_schema, field| Ok(field.clone()),
                ),
                UnaryOp::StringLowercase => input.str().to_lowercase(),
                UnaryOp::StringUppercase => input.str().to_uppercase(),
                UnaryOp::StringHexEncode => input.str().hex_encode(),
                UnaryOp::StringBase64Encode => input.str().base64_encode(),
                UnaryOp::StringReverse => input.str().reverse(),
                UnaryOp::StringEscapeRegex => input.str().escape_regex(),
                UnaryOp::NullCount => input.null_count(),
                UnaryOp::DropNulls => input.drop_nulls(),
                UnaryOp::DropNans => input.drop_nans(),
                UnaryOp::FillNullForward => {
                    input.fill_null_with_strategy(polars::prelude::FillNullStrategy::Forward(None))
                }
                UnaryOp::FillNullBackward => {
                    input.fill_null_with_strategy(polars::prelude::FillNullStrategy::Backward(None))
                }
                UnaryOp::Interpolate => {
                    input.interpolate(polars::prelude::InterpolationMethod::Linear)
                }
                UnaryOp::AggGroups => {
                    return Err(TerlanPolarsError::new(
                        "unsupported_expression",
                        "agg_groups was removed in Polars 2.0; add a row index before grouping and aggregate that index column",
                    ));
                }
                UnaryOp::PeakMin => input.peak_min(),
                UnaryOp::PeakMax => input.peak_max(),
                UnaryOp::Rle => input.rle(),
                UnaryOp::RleId => input.rle_id(),
                UnaryOp::Sum => input.sum(),
                UnaryOp::Product => input.product(),
                UnaryOp::Count => input.count(),
                UnaryOp::Len => input.len(),
                UnaryOp::FirstNonNull => input.first_non_null(),
                UnaryOp::LastNonNull => input.last_non_null(),
                UnaryOp::Unique => input.unique(),
                UnaryOp::ArgUnique => input.arg_unique(),
                UnaryOp::ArgMin => input.arg_min(),
                UnaryOp::ArgMax => input.arg_max(),
                // Expression-level rechunking was removed in Polars 2.0. It
                // does not change values, so preserve the expression and let
                // callers rechunk the collected DataFrame when required.
                UnaryOp::Rechunk => input,
                UnaryOp::Reverse => input.reverse(),
                UnaryOp::Floor => input.floor(),
                UnaryOp::Ceil => input.ceil(),
                UnaryOp::Abs => input.abs(),
                UnaryOp::Neg => -input,
                UnaryOp::Sqrt => input.sqrt(),
                UnaryOp::Cbrt => input.cbrt(),
                UnaryOp::Sign => input.sign(),
                UnaryOp::ToPhysical => input.to_physical(),
                UnaryOp::ArgTrue => polars::lazy::dsl::arg_where(input),
                UnaryOp::Log1p => require_numeric(input, "log1p").log1p(),
                UnaryOp::Exp => require_numeric(input, "exp").exp(),
                UnaryOp::UpperBound => input.upper_bound(),
                UnaryOp::LowerBound => input.lower_bound(),
                UnaryOp::First => input.first(),
                UnaryOp::Last => input.last(),
                UnaryOp::SortAscending => input.sort(Default::default()),
                UnaryOp::RankDenseDescending => input.rank(
                    polars::prelude::RankOptions {
                        method: polars::prelude::RankMethod::Dense,
                        descending: true,
                    },
                    None,
                ),
                UnaryOp::Explode => input.explode(polars::prelude::ExplodeOptions {
                    empty_as_null: false,
                    keep_nulls: false,
                }),
                UnaryOp::ListLen => input.list().len(),
                UnaryOp::ListSum => input.list().sum(),
                UnaryOp::ListMean => input.list().mean(),
                UnaryOp::ListMin => input.list().min(),
                UnaryOp::ListMax => input.list().max(),
                UnaryOp::ListSort => input.list().sort(Default::default()),
                UnaryOp::ListMedian => input.list().median(),
                UnaryOp::ListFirst => input.list().first(),
                UnaryOp::ListLast => input.list().last(),
                UnaryOp::ListArgMin => input.list().arg_min(),
                UnaryOp::ListArgMax => input.list().arg_max(),
                UnaryOp::ListDropNulls => input.list().drop_nulls(),
                UnaryOp::ListToMap => input.list().to_map(),
                UnaryOp::BitwiseCountOnes => input.bitwise_count_ones(),
                UnaryOp::BitwiseCountZeros => input.bitwise_count_zeros(),
                UnaryOp::BitwiseLeadingOnes => input.bitwise_leading_ones(),
                UnaryOp::BitwiseLeadingZeros => input.bitwise_leading_zeros(),
                UnaryOp::BitwiseTrailingOnes => input.bitwise_trailing_ones(),
                UnaryOp::BitwiseTrailingZeros => input.bitwise_trailing_zeros(),
                UnaryOp::BitwiseAnd => input.bitwise_and(),
                UnaryOp::BitwiseOr => input.bitwise_or(),
                UnaryOp::BitwiseXor => input.bitwise_xor(),
                UnaryOp::ArrayLen => input.arr().len(),
                UnaryOp::ArraySum => input.arr().sum(),
                UnaryOp::ArrayMean => input.arr().mean(),
                UnaryOp::ArrayMin => input.arr().min(),
                UnaryOp::ArrayMax => input.arr().max(),
                UnaryOp::ArraySort => input.arr().sort(Default::default()),
                UnaryOp::ArrayToList => input.arr().to_list(),
                UnaryOp::ArrayMedian => input.arr().median(),
                UnaryOp::ArrayArgMin => input.arr().arg_min(),
                UnaryOp::ArrayArgMax => input.arr().arg_max(),
                UnaryOp::ArrayToStruct => {
                    preserve_outer_validity(input.clone().arr().to_struct(None), input)
                }
                UnaryOp::MapEntries => input.map_().entries(),
                UnaryOp::MapKeys => input.map_().keys(),
                UnaryOp::MapValues => input.map_().values(),
                UnaryOp::MapLen => input.map_().len(),
                UnaryOp::CategoricalCategories => input.unique(),
                UnaryOp::CategoricalLenBytes => input.cat().len_bytes(),
                UnaryOp::CategoricalLenChars => input.cat().len_chars(),
                UnaryOp::BinarySizeBytes => input.binary().size_bytes(),
                UnaryOp::BinaryHexEncode => input.binary().hex_encode(),
                UnaryOp::BinaryBase64Encode => input.binary().base64_encode(),
                UnaryOp::StructJsonEncode => input.struct_().json_encode(),
                UnaryOp::Cos => input.cos(),
                UnaryOp::Cot => input.cot(),
                UnaryOp::Sin => input.sin(),
                UnaryOp::Tan => input.tan(),
                UnaryOp::ArcCos => input.arccos(),
                UnaryOp::ArcSin => input.arcsin(),
                UnaryOp::ArcTan => input.arctan(),
                UnaryOp::Cosh => input.cosh(),
                UnaryOp::Sinh => input.sinh(),
                UnaryOp::Tanh => input.tanh(),
                UnaryOp::ArcCosh => input.arccosh(),
                UnaryOp::ArcSinh => input.arcsinh(),
                UnaryOp::ArcTanh => input.arctanh(),
                UnaryOp::Degrees => input.degrees(),
                UnaryOp::Radians => input.radians(),
            }
        }
        ExprNode::Binary { kind, left, right } => {
            let left = compile(left)?;
            let right = compile(right)?;
            match kind {
                BinaryOp::Add => left + right,
                BinaryOp::Subtract => left - right,
                BinaryOp::Multiply => left * right,
                BinaryOp::Divide => left / right,
                BinaryOp::TrueDivide => left.true_div(right),
                BinaryOp::FloorDivide => left.floor_div(right),
                BinaryOp::Modulo => left % right,
                BinaryOp::Pow => left.pow(right),
                BinaryOp::Log => require_numeric(left, "log").log(right),
                BinaryOp::Dot => left.dot(right),
                BinaryOp::IndexOf => left.index_of(right),
                BinaryOp::Shift => left.shift(right),
                BinaryOp::RepeatBy => left.repeat_by(right),
                BinaryOp::PctChange => left.pct_change(right),
                BinaryOp::Equal => left.eq(right),
                BinaryOp::NotEqual => left.neq(right),
                BinaryOp::EqualMissing => left.eq_missing(right),
                BinaryOp::NotEqualMissing => left.neq_missing(right),
                BinaryOp::LessThan => left.lt(right),
                BinaryOp::LessThanOrEqual => left.lt_eq(right),
                BinaryOp::GreaterThan => left.gt(right),
                BinaryOp::GreaterThanOrEqual => left.gt_eq(right),
                BinaryOp::And => reject_boolean_integer_pair(left, right.clone(), "&").and(right),
                BinaryOp::Or => reject_boolean_integer_pair(left, right.clone(), "|").or(right),
                BinaryOp::Xor => reject_boolean_integer_pair(left, right.clone(), "^").xor(right),
                BinaryOp::LogicalAnd => left.logical_and(right),
                BinaryOp::LogicalOr => left.logical_or(right),
                BinaryOp::StringStartsWith => left.str().starts_with(right),
                BinaryOp::StringContains => left.str().contains(right, true),
                BinaryOp::StringContainsLiteral => left.str().contains_literal(right),
                BinaryOp::StringEndsWith => left.str().ends_with(right),
                BinaryOp::StringStripChars => left.str().strip_chars(right),
                BinaryOp::StringStripCharsStart => left.str().strip_chars_start(right),
                BinaryOp::StringStripCharsEnd => left.str().strip_chars_end(right),
                BinaryOp::StringStripPrefix => left.str().strip_prefix(right),
                BinaryOp::StringStripSuffix => left.str().strip_suffix(right),
                BinaryOp::FillNull => left.fill_null(right),
                BinaryOp::FillNan => left.fill_nan(right),
                BinaryOp::InterpolateBy => left.interpolate_by(right),
                BinaryOp::ListGet => left.list().get(right, true),
                BinaryOp::ListContains => validated_membership_values(left, right.clone())
                    .list()
                    .contains(right, true),
                BinaryOp::ListShift => left.list().shift(right),
                BinaryOp::ListCountMatches => left.list().count_matches(right),
                BinaryOp::ListSetUnion => left.list().union(right),
                BinaryOp::ListSetDifference => left.list().set_difference(right),
                BinaryOp::ListSetIntersection => left.list().set_intersection(right),
                BinaryOp::ListSetSymmetricDifference => left.list().set_symmetric_difference(right),
                BinaryOp::ArrayGet => left.arr().get(right, true),
                BinaryOp::ArrayContains => validated_membership_values(left, right.clone())
                    .arr()
                    .contains(right, true),
                BinaryOp::ArrayCountMatches => left.arr().count_matches(right),
                BinaryOp::ArrayShift => left.arr().shift(right),
                BinaryOp::MapContainsKey => left.map_().contains_key(right),
                BinaryOp::MapGet => left.map_().get(right),
                BinaryOp::BinaryContains => left.binary().contains_literal(right),
                BinaryOp::BinaryStartsWith => left.binary().starts_with(right),
                BinaryOp::BinaryEndsWith => left.binary().ends_with(right),
                BinaryOp::BinaryGet => left.binary().get(right, true),
                BinaryOp::BinaryHead => left.binary().head(right),
                BinaryOp::BinaryTail => left.binary().tail(right),
                BinaryOp::ArcTan2 => left.arctan2(right),
            }
        }
        ExprNode::Alias { input, name } => compile(input)?.alias(name),
        ExprNode::Rename {
            input,
            value,
            prefix,
        } => {
            let input = compile(input)?;
            if *prefix {
                input.name().prefix(value)
            } else {
                input.name().suffix(value)
            }
        }
        ExprNode::KeepName { input } => compile(input)?.name().keep(),
        ExprNode::ReplaceName {
            input,
            pattern,
            value,
            literal,
        } => compile(input)?.name().replace(pattern, value, *literal),
        ExprNode::LowercaseNames { input } => compile(input)?.name().to_lowercase(),
        ExprNode::MapNames {
            input,
            fields,
            transform,
        } => {
            let transform = transform.clone();
            let callback: polars::prelude::PlanCallback<
                polars::prelude::PlSmallStr,
                polars::prelude::PlSmallStr,
            > = polars::prelude::PlanCallback::new(move |name: polars::prelude::PlSmallStr| {
                Ok(polars::prelude::PlSmallStr::from_string(transformed_name(
                    name.as_str(),
                    &transform,
                )))
            });
            let input = compile(input)?;
            if *fields {
                input.name().map_fields(callback)
            } else {
                input.name().map(callback)
            }
        }
        ExprNode::StructFieldAffix {
            input,
            value,
            prefix,
        } => {
            let input = compile(input)?;
            if *prefix {
                input.name().prefix_fields(value)
            } else {
                input.name().suffix_fields(value)
            }
        }
        ExprNode::CategoricalAffix {
            input,
            value,
            prefix,
        } => {
            let input = compile(input)?;
            if *prefix {
                input.cat().starts_with(value.clone())
            } else {
                input.cat().ends_with(value.clone())
            }
        }
        ExprNode::CategoricalSlice {
            input,
            offset,
            length,
        } => compile(input)?.cat().slice(*offset, *length),
        ExprNode::BinarySlice {
            input,
            offset,
            length,
        } => compile(input)?
            .binary()
            .slice(compile(offset)?, compile(length)?),
        ExprNode::BinaryDecode {
            input,
            encoding,
            strict,
        } => {
            let input = compile(input)?.binary();
            match encoding {
                BinaryEncoding::Hex => input.hex_decode(*strict),
                BinaryEncoding::Base64 => input.base64_decode(*strict),
            }
        }
        ExprNode::BinaryReinterpret {
            input,
            data_type,
            is_little_endian,
        } => compile(input)?
            .binary()
            .reinterpret(parse_data_type(data_type)?, *is_little_endian),
        ExprNode::ArrayStatistic {
            input,
            ddof,
            variance,
        } => {
            let input = compile(input)?.arr();
            if *variance {
                input.var(*ddof)
            } else {
                input.std(*ddof)
            }
        }
        ExprNode::ArrayJoin {
            input,
            separator,
            ignore_nulls,
        } => compile(input)?
            .arr()
            .join(compile(separator)?, *ignore_nulls),
        ExprNode::ArrayWindow {
            input,
            first,
            second,
            kind,
            as_array,
        } => {
            let input = compile(input)?.arr();
            match kind {
                ArrayWindowKind::Slice => {
                    let length = second.as_ref().ok_or_else(|| {
                        TerlanPolarsError::new(
                            "invalid_expression",
                            "array slice expression is missing its length",
                        )
                    })?;
                    input.slice(compile(first)?, compile(length)?, *as_array)?
                }
                ArrayWindowKind::Head => input.head(compile(first)?, *as_array)?,
                ArrayWindowKind::Tail => input.tail(compile(first)?, *as_array)?,
            }
        }
        ExprNode::ArrayExplode {
            input,
            empty_as_null,
            keep_nulls,
        } => compile(input)?
            .arr()
            .explode(polars::prelude::ExplodeOptions {
                empty_as_null: *empty_as_null,
                keep_nulls: *keep_nulls,
            }),
        ExprNode::ArrayEval {
            input,
            evaluation,
            as_list,
            aggregate,
        } => {
            let input = compile(input)?.arr();
            let evaluation = compile(evaluation)?;
            if *aggregate {
                input.agg(evaluation)
            } else {
                input.eval(evaluation, *as_list)
            }
        }
        ExprNode::ArraySortOptions {
            input,
            descending,
            nulls_last,
            maintain_order,
        } => compile(input)?.arr().sort(polars::prelude::SortOptions {
            descending: *descending,
            nulls_last: *nulls_last,
            multithreaded: true,
            maintain_order: *maintain_order,
            limit: None,
        }),
        ExprNode::ArrayBinaryOptions {
            input,
            value,
            kind,
            option,
        } => {
            let input = compile(input)?.arr();
            let value = compile(value)?;
            match kind {
                ArrayBinaryOptionsKind::Get => input.get(value, *option),
                ArrayBinaryOptionsKind::Contains => input.contains(value, *option),
            }
        }
        ExprNode::ListStatistic {
            input,
            ddof,
            variance,
        } => {
            let input = compile(input)?.list();
            if *variance {
                input.var(*ddof)
            } else {
                input.std(*ddof)
            }
        }
        ExprNode::ListJoin {
            input,
            separator,
            ignore_nulls,
        } => compile(input)?
            .list()
            .join(compile(separator)?, *ignore_nulls),
        ExprNode::ListWindow {
            input,
            first,
            second,
            kind,
        } => {
            let input = compile(input)?.list();
            match kind {
                ListWindowKind::Slice => {
                    let length = second.as_ref().ok_or_else(|| {
                        TerlanPolarsError::new(
                            "invalid_expression",
                            "list slice expression is missing its length",
                        )
                    })?;
                    input.slice(compile(first)?, compile(length)?)
                }
                ListWindowKind::Head => input.head(compile(first)?),
                ListWindowKind::Tail => input.tail(compile(first)?),
            }
        }
        ExprNode::ListToArray { input, width } => compile(input)?.list().to_array(*width),
        ExprNode::ListEval {
            input,
            evaluation,
            aggregate,
        } => {
            let input = compile(input)?.list();
            let evaluation = compile(evaluation)?;
            if *aggregate {
                input.agg(evaluation)
            } else {
                input.eval(evaluation)
            }
        }
        ExprNode::ListSortOptions {
            input,
            descending,
            nulls_last,
            maintain_order,
        } => compile(input)?.list().sort(polars::prelude::SortOptions {
            descending: *descending,
            nulls_last: *nulls_last,
            multithreaded: true,
            maintain_order: *maintain_order,
            limit: None,
        }),
        ExprNode::ListBinaryOptions {
            input,
            value,
            kind,
            option,
        } => {
            let input = compile(input)?.list();
            let value = compile(value)?;
            match kind {
                ListBinaryOptionsKind::Get => input.get(value, *option),
                ListBinaryOptionsKind::Contains => input.contains(value, *option),
            }
        }
        ExprNode::ListSample {
            input,
            amount,
            fraction,
            with_replacement,
            shuffle,
            seed,
        } => {
            let input = compile(input)?.list();
            let amount = compile(amount)?;
            if *fraction {
                input.sample_fraction(amount, *with_replacement, Some(*shuffle), *seed)
            } else {
                input.sample_n(amount, *with_replacement, Some(*shuffle), *seed)
            }
        }
        ExprNode::ListGather {
            input,
            index,
            offset,
            null_on_oob,
        } => {
            let input = compile(input)?.list();
            if let Some(offset) = offset {
                input.gather_every(compile(index)?, compile(offset)?)
            } else {
                input.gather(require_list(compile(index)?, "list.gather"), *null_on_oob)
            }
        }
        ExprNode::ListDiff {
            input,
            periods,
            drop_nulls,
        } => compile(input)?.list().diff(
            *periods,
            if *drop_nulls {
                polars::series::ops::NullBehavior::Drop
            } else {
                polars::series::ops::NullBehavior::Ignore
            },
        ),
        ExprNode::ListToStruct { input, names } => {
            let input = compile(input)?;
            let structure = input.clone().list().to_struct(
                names
                    .iter()
                    .map(|name| name.as_str().into())
                    .collect::<Vec<_>>()
                    .into(),
            );
            preserve_outer_validity(structure, input)
        }
        ExprNode::Random {
            input,
            amount,
            fraction,
            with_replacement,
            shuffle,
            seed,
        } => {
            let input = compile(input)?;
            match amount {
                None => input.shuffle(*seed),
                Some(amount) if *fraction => {
                    input.sample_frac(compile(amount)?, *with_replacement, Some(*shuffle), *seed)
                }
                Some(amount) => {
                    input.sample_n(compile(amount)?, *with_replacement, Some(*shuffle), *seed)
                }
            }
        }
        ExprNode::Hash { input, seed } => {
            use polars::prelude::{
                DataType, Field, PlSeedableRandomStateQuality, SeedableFromU64SeedExt,
                SeriesMethods,
            };

            let seed = *seed;
            compile(input)?.map(
                move |column| {
                    Ok(column
                        .as_materialized_series()
                        .hash(PlSeedableRandomStateQuality::seed_from_u64(seed))
                        .into_column())
                },
                |_schema, field| Ok(Field::new(field.name().clone(), DataType::UInt64)),
            )
        }
        ExprNode::Statistic {
            input,
            by,
            kind,
            ddof,
        } => {
            let input = compile(input)?;
            match kind {
                StatisticKind::Std => reject_duration(input, "std").std(ddof.ok_or_else(|| {
                    TerlanPolarsError::new(
                        "invalid_expression",
                        "standard deviation expression is missing degrees of freedom",
                    )
                })?),
                StatisticKind::Var => reject_duration(input, "var").var(ddof.ok_or_else(|| {
                    TerlanPolarsError::new(
                        "invalid_expression",
                        "variance expression is missing degrees of freedom",
                    )
                })?),
                StatisticKind::Min => input.min(),
                StatisticKind::Median => input.median(),
                StatisticKind::MinBy => input.min_by(compile(by.as_ref().ok_or_else(|| {
                    TerlanPolarsError::new(
                        "invalid_expression",
                        "min-by expression is missing its ordering expression",
                    )
                })?)?),
                StatisticKind::MaxBy => input.max_by(compile(by.as_ref().ok_or_else(|| {
                    TerlanPolarsError::new(
                        "invalid_expression",
                        "max-by expression is missing its ordering expression",
                    )
                })?)?),
                StatisticKind::NanMin => input.nan_min(),
                StatisticKind::NanMax => input.nan_max(),
            }
        }
        ExprNode::Item { input, allow_empty } => compile(input)?.item(*allow_empty),
        ExprNode::Implode {
            input,
            maintain_order,
        } => compile(input)?.implode(*maintain_order),
        ExprNode::Quantile {
            input,
            probability,
            method,
        } => compile(input)?.quantile(
            compile(probability)?,
            match method {
                RollingQuantileMethod::Nearest => polars::prelude::QuantileMethod::Nearest,
                RollingQuantileMethod::Lower => polars::prelude::QuantileMethod::Lower,
                RollingQuantileMethod::Higher => polars::prelude::QuantileMethod::Higher,
                RollingQuantileMethod::Midpoint => polars::prelude::QuantileMethod::Midpoint,
                RollingQuantileMethod::Linear => polars::prelude::QuantileMethod::Linear,
                RollingQuantileMethod::Equiprobable => {
                    polars::prelude::QuantileMethod::Equiprobable
                }
            },
        ),
        ExprNode::Mode {
            input,
            maintain_order,
        } => compile(input)?.mode(*maintain_order),
        ExprNode::Rank {
            input,
            method,
            descending,
            seed,
        } => compile(input)?.rank(
            polars::prelude::RankOptions {
                method: match method {
                    RankKind::Average => polars::prelude::RankMethod::Average,
                    RankKind::Min => polars::prelude::RankMethod::Min,
                    RankKind::Max => polars::prelude::RankMethod::Max,
                    RankKind::Dense => polars::prelude::RankMethod::Dense,
                    RankKind::Ordinal => polars::prelude::RankMethod::Ordinal,
                    RankKind::Random => polars::prelude::RankMethod::Random,
                },
                descending: *descending,
            },
            *seed,
        ),
        ExprNode::TopK {
            input,
            count,
            bottom,
        } => {
            let input = compile(input)?;
            let count = compile(count)?;
            if *bottom {
                input.bottom_k(count)
            } else {
                input.top_k(count)
            }
        }
        ExprNode::TopKBy {
            input,
            count,
            by,
            descending,
            bottom,
        } => {
            let input = compile(input)?;
            let count = compile(count)?;
            let by = by.iter().map(compile).collect::<Result<Vec<_>, _>>()?;
            if *bottom {
                input.bottom_k_by(count, by, descending.clone())
            } else {
                input.top_k_by(count, by, descending.clone())
            }
        }
        ExprNode::ReplaceValues {
            input,
            old,
            new,
            default,
            strict,
        } => {
            let input = compile(input)?;
            let old = compile(old)?;
            let new = compile(new)?;
            if *strict {
                let default = default.as_deref().map(compile).transpose()?;
                input.replace_strict(old, new, default, None::<polars::prelude::DataType>)
            } else {
                input.replace(old, new)
            }
        }
        ExprNode::Cut {
            input,
            values,
            labels,
            kind,
            left_closed,
            allow_duplicates,
            include_breaks,
        } => {
            let input = compile(input)?;
            let labels = labels.as_ref().map(|labels| {
                labels
                    .iter()
                    .map(|label| label.as_str().into())
                    .collect::<Vec<polars::prelude::PlSmallStr>>()
            });
            match kind {
                CutKind::Breaks => input.cut(values.clone(), labels, *left_closed, *include_breaks),
                CutKind::Quantiles => input.qcut(
                    values.clone(),
                    labels,
                    *left_closed,
                    *allow_duplicates,
                    *include_breaks,
                ),
                CutKind::Uniform { bins } => input.qcut_uniform(
                    *bins,
                    labels,
                    *left_closed,
                    *allow_duplicates,
                    *include_breaks,
                ),
            }
        }
        ExprNode::Bin {
            input,
            method,
            labels,
            include_intervals,
        } => {
            use polars_plan::dsl::{BinMethod, BinOptions, DslIntervalSpec, FractionSpec};

            let input = compile(input)?;
            let labels = labels.as_ref().map(|labels| {
                labels
                    .iter()
                    .map(|label| label.as_str().into())
                    .collect::<Vec<polars::prelude::PlSmallStr>>()
            });
            let method = match method {
                BinMethodKind::Intervals { spec, right_closed } => {
                    let spec = match spec {
                        IntervalBinSpec::Floats(values) => DslIntervalSpec::from_breaks(
                            Series::new("breaks".into(), values.clone()),
                        ),
                        IntervalBinSpec::Ints(values) => DslIntervalSpec::from_breaks(Series::new(
                            "breaks".into(),
                            values.clone(),
                        )),
                        IntervalBinSpec::Strings(values) => DslIntervalSpec::from_breaks(
                            Series::new("breaks".into(), values.clone()),
                        ),
                        IntervalBinSpec::Bools(values) => DslIntervalSpec::from_breaks(
                            Series::new("breaks".into(), values.clone()),
                        ),
                        IntervalBinSpec::Series { ipc_base64, .. } => {
                            DslIntervalSpec::from_breaks(series_from_ipc_payload(ipc_base64)?)
                        }
                        IntervalBinSpec::Count(count) => DslIntervalSpec::from_count(*count)?,
                    };
                    BinMethod::Intervals {
                        spec,
                        right_closed: *right_closed,
                    }
                }
                BinMethodKind::Quantiles { spec, right_closed } => {
                    let spec = match spec {
                        FractionBinSpec::Fractions(values) => {
                            FractionSpec::from_fractions(values.clone())?
                        }
                        FractionBinSpec::Count(count) => FractionSpec::from_count(*count)?,
                    };
                    BinMethod::Quantiles {
                        spec,
                        right_closed: *right_closed,
                    }
                }
                BinMethodKind::Ranks { spec } => {
                    let spec = match spec {
                        FractionBinSpec::Fractions(values) => {
                            FractionSpec::from_fractions(values.clone())?
                        }
                        FractionBinSpec::Count(count) => FractionSpec::from_count(*count)?,
                    };
                    BinMethod::Ranks { spec }
                }
            };
            input.bin(BinOptions {
                method,
                labels,
                include_intervals: *include_intervals,
            })
        }
        ExprNode::Reshape { input, dimensions } => compile(input)?.reshape(dimensions),
        ExprNode::Ewm {
            input,
            kind,
            alpha,
            adjust,
            bias,
            min_samples,
            ignore_nulls,
        } => {
            let options = polars::prelude::EWMOptions {
                alpha: *alpha,
                adjust: *adjust,
                bias: *bias,
                min_periods: *min_samples,
                ignore_nulls: *ignore_nulls,
            };
            let input = compile(input)?;
            match kind {
                EwmKind::Sum => input.ewm_sum(options),
                EwmKind::Mean => input.ewm_mean(options),
                EwmKind::StandardDeviation => reject_duration(input, "ewm_std").ewm_std(options),
                EwmKind::Variance => reject_duration(input, "ewm_var").ewm_var(options),
            }
        }
        ExprNode::EwmBy {
            input,
            by,
            kind,
            half_life,
        } => {
            let half_life = polars::prelude::Duration::try_parse(half_life).map_err(|error| {
                TerlanPolarsError::new(
                    "invalid_expression",
                    format!("invalid exponentially weighted half-life: {error}"),
                )
            })?;
            if half_life.negative() || half_life.is_zero() || half_life.months() != 0 {
                return Err(TerlanPolarsError::new(
                    "invalid_expression",
                    "exponentially weighted half-life must be a positive constant duration",
                ));
            }
            let input = compile(input)?;
            let by = compile(by)?;
            match kind {
                EwmByKind::Sum => input.ewm_sum_by(by, half_life),
                EwmByKind::Mean => input.ewm_mean_by(by, half_life),
            }
        }
        ExprNode::Cumulative {
            input,
            kind,
            reverse,
        } => {
            let input = compile(input)?;
            match kind {
                CumulativeKind::Count => input.cum_count(*reverse),
                CumulativeKind::Sum => input.cum_sum(*reverse),
                CumulativeKind::Product => input.cum_prod(*reverse),
                CumulativeKind::Min => input.cum_min(*reverse),
                CumulativeKind::Max => input.cum_max(*reverse),
            }
        }
        ExprNode::CumulativeEval {
            input,
            evaluation,
            min_samples,
        } => compile(input)?.cumulative_eval(compile(evaluation)?, *min_samples),
        ExprNode::Rolling {
            input,
            kind,
            window_size,
            min_samples,
            center,
            weights,
        } => {
            let input = compile(input)?;
            let mut options = polars::prelude::RollingOptionsFixedWindow {
                window_size: *window_size,
                min_periods: *min_samples,
                center: *center,
                weights: weights.clone(),
                ..Default::default()
            };
            match kind {
                RollingKind::Min => input.rolling_min(options),
                RollingKind::Max => input.rolling_max(options),
                RollingKind::Mean => input.rolling_mean(options),
                RollingKind::Sum => input.rolling_sum(options),
                RollingKind::Median => input.rolling_median(options),
                RollingKind::Var => input.rolling_var(options),
                RollingKind::Std => input.rolling_std(options),
                RollingKind::Quantile {
                    probability,
                    method,
                } => {
                    let method = match method {
                        RollingQuantileMethod::Nearest => polars::prelude::QuantileMethod::Nearest,
                        RollingQuantileMethod::Lower => polars::prelude::QuantileMethod::Lower,
                        RollingQuantileMethod::Higher => polars::prelude::QuantileMethod::Higher,
                        RollingQuantileMethod::Midpoint => {
                            polars::prelude::QuantileMethod::Midpoint
                        }
                        RollingQuantileMethod::Linear => polars::prelude::QuantileMethod::Linear,
                        RollingQuantileMethod::Equiprobable => {
                            polars::prelude::QuantileMethod::Equiprobable
                        }
                    };
                    options.fn_params = Some(polars::prelude::RollingFnParams::Quantile(
                        polars::polars_compute::rolling::RollingQuantileParams {
                            prob: *probability,
                            method,
                        },
                    ));
                    input.rolling_quantile(method, *probability, options)
                }
                RollingKind::Rank { method, seed } => {
                    let method = match method {
                        RollingRankKind::Average => polars::prelude::RollingRankMethod::Average,
                        RollingRankKind::Min => polars::prelude::RollingRankMethod::Min,
                        RollingRankKind::Max => polars::prelude::RollingRankMethod::Max,
                        RollingRankKind::Dense => polars::prelude::RollingRankMethod::Dense,
                        RollingRankKind::Random => polars::prelude::RollingRankMethod::Random,
                    };
                    options.fn_params = Some(polars::prelude::RollingFnParams::Rank {
                        method,
                        seed: Some(*seed),
                    });
                    input.rolling_rank(options)
                }
                RollingKind::Skew { bias } => {
                    options.fn_params =
                        Some(polars::prelude::RollingFnParams::Skew { bias: *bias });
                    input.rolling_skew(options)
                }
                RollingKind::Kurtosis { fisher, bias } => {
                    options.fn_params = Some(polars::prelude::RollingFnParams::Kurtosis {
                        fisher: *fisher,
                        bias: *bias,
                    });
                    input.rolling_kurtosis(options)
                }
            }
        }
        ExprNode::RollingMap {
            input,
            udf,
            window_size,
            min_samples,
            center,
        } => {
            use polars::prelude::{DataFrame, IntoLazy, PlanCallback};

            let udf = udf.clone();
            let callback = PlanCallback::new(move |series: polars::prelude::Series| {
                let result = (|| -> Result<polars::prelude::Series, TerlanPolarsError> {
                    let argument = expr_series(&crate::TerlanPolarsSeries { inner: series })?;
                    let applied = crate::apply_expression_udf(&udf, &[argument])?;
                    let frame = DataFrame::empty()
                        .lazy()
                        .select([compile_one(&applied)?])
                        .collect()?;
                    let output = frame.select_at_idx(0).ok_or_else(|| {
                        TerlanPolarsError::new(
                            "invalid_udf",
                            "rolling map UDF produced no output column",
                        )
                    })?;
                    Ok(output.as_materialized_series().clone())
                })();
                result.map_err(|error| {
                    polars::error::PolarsError::ComputeError(
                        format!("{}: {}", error.code(), error.message()).into(),
                    )
                })
            });
            compile(input)?.rolling_map(
                callback,
                polars::prelude::RollingOptionsFixedWindow {
                    window_size: *window_size,
                    min_periods: *min_samples,
                    center: *center,
                    ..Default::default()
                },
            )
        }
        ExprNode::RollingGroup {
            input,
            index_column,
            period,
            offset,
            closed,
        } => {
            let period = polars::prelude::Duration::try_parse(period).map_err(|error| {
                TerlanPolarsError::new(
                    "invalid_expression",
                    format!("invalid rolling period: {error}"),
                )
            })?;
            let offset = polars::prelude::Duration::try_parse(offset).map_err(|error| {
                TerlanPolarsError::new(
                    "invalid_expression",
                    format!("invalid rolling offset: {error}"),
                )
            })?;
            let closed = match closed {
                ClosedIntervalKind::Both => polars::prelude::ClosedWindow::Both,
                ClosedIntervalKind::Left => polars::prelude::ClosedWindow::Left,
                ClosedIntervalKind::Right => polars::prelude::ClosedWindow::Right,
                ClosedIntervalKind::None => polars::prelude::ClosedWindow::None,
            };
            compile(input)?.rolling(compile(index_column)?, period, offset, closed)
        }
        ExprNode::RollingBy {
            input,
            by,
            kind,
            window_size,
            min_samples,
            closed,
        } => {
            let input = compile(input)?;
            let by = compile(by)?;
            let window_size =
                polars::prelude::Duration::try_parse(window_size).map_err(|error| {
                    TerlanPolarsError::new(
                        "invalid_expression",
                        format!("invalid rolling-by window duration: {error}"),
                    )
                })?;
            let closed_window = match closed {
                ClosedIntervalKind::Both => polars::prelude::ClosedWindow::Both,
                ClosedIntervalKind::Left => polars::prelude::ClosedWindow::Left,
                ClosedIntervalKind::Right => polars::prelude::ClosedWindow::Right,
                ClosedIntervalKind::None => polars::prelude::ClosedWindow::None,
            };
            let mut options = polars::prelude::RollingOptionsDynamicWindow {
                window_size,
                min_periods: *min_samples,
                closed_window,
                fn_params: None,
            };
            match kind {
                RollingKind::Min => input.rolling_min_by(by, options),
                RollingKind::Max => input.rolling_max_by(by, options),
                RollingKind::Mean => input.rolling_mean_by(by, options),
                RollingKind::Sum => input.rolling_sum_by(by, options),
                RollingKind::Median => input.rolling_median_by(by, options),
                RollingKind::Var => input.rolling_var_by(by, options),
                RollingKind::Std => input.rolling_std_by(by, options),
                RollingKind::Quantile {
                    probability,
                    method,
                } => {
                    let method = match method {
                        RollingQuantileMethod::Nearest => polars::prelude::QuantileMethod::Nearest,
                        RollingQuantileMethod::Lower => polars::prelude::QuantileMethod::Lower,
                        RollingQuantileMethod::Higher => polars::prelude::QuantileMethod::Higher,
                        RollingQuantileMethod::Midpoint => {
                            polars::prelude::QuantileMethod::Midpoint
                        }
                        RollingQuantileMethod::Linear => polars::prelude::QuantileMethod::Linear,
                        RollingQuantileMethod::Equiprobable => {
                            polars::prelude::QuantileMethod::Equiprobable
                        }
                    };
                    input.rolling_quantile_by(by, method, *probability, options)
                }
                RollingKind::Rank { method, seed } => {
                    let method = match method {
                        RollingRankKind::Average => polars::prelude::RollingRankMethod::Average,
                        RollingRankKind::Min => polars::prelude::RollingRankMethod::Min,
                        RollingRankKind::Max => polars::prelude::RollingRankMethod::Max,
                        RollingRankKind::Dense => polars::prelude::RollingRankMethod::Dense,
                        RollingRankKind::Random => polars::prelude::RollingRankMethod::Random,
                    };
                    options.fn_params = Some(polars::prelude::RollingFnParams::Rank {
                        method,
                        seed: Some(*seed),
                    });
                    input.rolling_rank_by(by, options)
                }
                RollingKind::Skew { .. } | RollingKind::Kurtosis { .. } => {
                    return Err(TerlanPolarsError::new(
                        "invalid_expression",
                        "skewness and kurtosis do not have rolling-by operations in pinned Polars",
                    ));
                }
            }
        }
        ExprNode::Histogram {
            input,
            bins,
            bin_count,
            include_category,
            include_breakpoint,
        } => compile(input)?.hist(
            bins.as_ref().map(|bins| compile(bins)).transpose()?,
            *bin_count,
            *include_category,
            *include_breakpoint,
        ),
        ExprNode::Timestamp { input, unit } => compile(input)?.dt().timestamp(match unit {
            TemporalTimeUnit::Milliseconds => polars::prelude::TimeUnit::Milliseconds,
            TemporalTimeUnit::Microseconds => polars::prelude::TimeUnit::Microseconds,
            TemporalTimeUnit::Nanoseconds => polars::prelude::TimeUnit::Nanoseconds,
        }),
        ExprNode::TemporalTransform { input, value, kind } => {
            let input = compile(input)?.dt();
            let value = compile(value)?;
            match kind {
                TemporalTransformKind::Truncate => input.truncate(value),
                TemporalTransformKind::Round => input.round(value),
                TemporalTransformKind::OffsetBy => input.offset_by(value),
            }
        }
        ExprNode::TemporalRange {
            start,
            end,
            interval,
            closed,
            kind,
            time_unit,
            time_zone,
            ranges,
        } => {
            let interval = polars::prelude::Duration::try_parse(interval).map_err(|error| {
                TerlanPolarsError::new(
                    "invalid_expression",
                    format!("invalid temporal range duration: {error}"),
                )
            })?;
            let closed = match closed {
                ClosedIntervalKind::Both => polars::prelude::ClosedWindow::Both,
                ClosedIntervalKind::Left => polars::prelude::ClosedWindow::Left,
                ClosedIntervalKind::Right => polars::prelude::ClosedWindow::Right,
                ClosedIntervalKind::None => polars::prelude::ClosedWindow::None,
            };
            let start = compile(start)?;
            let end = compile(end)?;
            match kind {
                TemporalRangeKind::Date if *ranges => polars::lazy::dsl::date_ranges(
                    Some(start),
                    Some(end),
                    Some(interval),
                    None,
                    closed,
                )?,
                TemporalRangeKind::Date => polars::lazy::dsl::date_range(
                    Some(start),
                    Some(end),
                    Some(interval),
                    None,
                    closed,
                )?,
                TemporalRangeKind::Datetime => {
                    let time_unit = time_unit.as_ref().map(|unit| match unit {
                        TemporalTimeUnit::Milliseconds => TimeUnit::Milliseconds,
                        TemporalTimeUnit::Microseconds => TimeUnit::Microseconds,
                        TemporalTimeUnit::Nanoseconds => TimeUnit::Nanoseconds,
                    });
                    let time_zone = polars::prelude::TimeZone::opt_try_new(
                        (!time_zone.is_empty()).then(|| time_zone.clone()),
                    )?;
                    if *ranges {
                        polars::lazy::dsl::datetime_ranges(
                            Some(start),
                            Some(end),
                            Some(interval),
                            None,
                            closed,
                            time_unit,
                            time_zone,
                        )?
                    } else {
                        polars::lazy::dsl::datetime_range(
                            Some(start),
                            Some(end),
                            Some(interval),
                            None,
                            closed,
                            time_unit,
                            time_zone,
                        )?
                    }
                }
                TemporalRangeKind::Time if *ranges => {
                    polars::lazy::dsl::time_ranges(start, end, interval, closed)
                }
                TemporalRangeKind::Time => {
                    polars::lazy::dsl::time_range(start, end, interval, closed)
                }
            }
        }
        ExprNode::Duration { values, unit } => {
            if values.len() != 8 {
                return Err(TerlanPolarsError::new(
                    "invalid_expression",
                    "duration descriptor must contain eight component expressions",
                ));
            }
            let values = values.iter().map(compile).collect::<Result<Vec<_>, _>>()?;
            polars::lazy::dsl::duration(polars::lazy::dsl::DurationArgs {
                weeks: values[0].clone(),
                days: values[1].clone(),
                hours: values[2].clone(),
                minutes: values[3].clone(),
                seconds: values[4].clone(),
                milliseconds: values[5].clone(),
                microseconds: values[6].clone(),
                nanoseconds: values[7].clone(),
                time_unit: match unit {
                    TemporalTimeUnit::Milliseconds => TimeUnit::Milliseconds,
                    TemporalTimeUnit::Microseconds => TimeUnit::Microseconds,
                    TemporalTimeUnit::Nanoseconds => TimeUnit::Nanoseconds,
                },
            })
        }
        ExprNode::DatetimeParts {
            values,
            unit,
            time_zone,
        } => {
            if values.len() != 8 {
                return Err(TerlanPolarsError::new(
                    "invalid_expression",
                    "datetime descriptor must contain eight component expressions",
                ));
            }
            let values = values.iter().map(compile).collect::<Result<Vec<_>, _>>()?;
            let time_zone = polars::prelude::TimeZone::opt_try_new(
                (!time_zone.is_empty()).then(|| time_zone.clone()),
            )?;
            polars::lazy::dsl::datetime(polars::lazy::dsl::DatetimeArgs {
                year: values[0].clone(),
                month: values[1].clone(),
                day: values[2].clone(),
                hour: values[3].clone(),
                minute: values[4].clone(),
                second: values[5].clone(),
                microsecond: values[6].clone(),
                ambiguous: values[7].clone(),
                time_unit: match unit {
                    TemporalTimeUnit::Milliseconds => TimeUnit::Milliseconds,
                    TemporalTimeUnit::Microseconds => TimeUnit::Microseconds,
                    TemporalTimeUnit::Nanoseconds => TimeUnit::Nanoseconds,
                },
                time_zone,
            })
        }
        ExprNode::Repeat { value, count } => {
            polars::lazy::dsl::repeat(compile(value)?, compile(count)?)
        }
        ExprNode::TemporalReplace { input, values } => {
            if values.len() != 8 {
                return Err(TerlanPolarsError::new(
                    "invalid_expression",
                    "temporal replace descriptor must contain eight expressions",
                ));
            }
            let values = values.iter().map(compile).collect::<Result<Vec<_>, _>>()?;
            compile(input)?.dt().replace(
                values[0].clone(),
                values[1].clone(),
                values[2].clone(),
                values[3].clone(),
                values[4].clone(),
                values[5].clone(),
                values[6].clone(),
                values[7].clone(),
            )
        }
        ExprNode::ConvertTimeZone { input, time_zone } => {
            let time_zone = polars::prelude::TimeZone::opt_try_new(Some(time_zone.clone()))?
                .ok_or_else(|| {
                    TerlanPolarsError::new(
                        "invalid_expression",
                        "conversion time zone cannot be empty",
                    )
                })?;
            compile(input)?.dt().convert_time_zone(time_zone)
        }
        ExprNode::ReplaceTimeZone {
            input,
            time_zone,
            ambiguous,
            non_existent,
        } => {
            let time_zone = polars::prelude::TimeZone::opt_try_new(
                (!time_zone.is_empty()).then(|| time_zone.clone()),
            )?;
            let non_existent = match non_existent.as_str() {
                "raise" => polars::prelude::NonExistent::Raise,
                "null" => polars::prelude::NonExistent::Null,
                _ => {
                    return Err(TerlanPolarsError::new(
                        "invalid_expression",
                        "non-existent time policy must be raise or null",
                    ));
                }
            };
            compile(input)?
                .dt()
                .replace_time_zone(time_zone, compile(ambiguous)?, non_existent)
        }
        ExprNode::DurationTotal {
            input,
            unit,
            fractional,
        } => {
            let input = compile(input)?.dt();
            match unit {
                DurationTotalUnit::Days => input.total_days(*fractional),
                DurationTotalUnit::Hours => input.total_hours(*fractional),
                DurationTotalUnit::Minutes => input.total_minutes(*fractional),
                DurationTotalUnit::Seconds => input.total_seconds(*fractional),
                DurationTotalUnit::Milliseconds => input.total_milliseconds(*fractional),
                DurationTotalUnit::Microseconds => input.total_microseconds(*fractional),
                DurationTotalUnit::Nanoseconds => input.total_nanoseconds(*fractional),
            }
        }
        ExprNode::Exclude { input, names } => match input.as_ref() {
            ExprNode::All => all().exclude_cols(names.clone()).as_expr(),
            ExprNode::Columns { names: columns } => {
                cols(columns.clone()).exclude_cols(names.clone()).as_expr()
            }
            ExprNode::DTypeColumns { data_types } => dtype_cols(parse_data_types(data_types)?)
                .as_selector()
                .exclude_cols(names.clone())
                .as_expr(),
            _ => {
                return Err(TerlanPolarsError::new(
                    "invalid_expression",
                    "exclude requires all_columns() or cols()",
                ))
            }
        },
        ExprNode::Round {
            input,
            decimals,
            mode,
        } => compile(input)?.round(
            *decimals,
            match mode {
                NumericRoundMode::HalfToEven => RoundMode::HalfToEven,
                NumericRoundMode::HalfAwayFromZero => RoundMode::HalfAwayFromZero,
                NumericRoundMode::ToZero => RoundMode::ToZero,
            },
        ),
        ExprNode::RoundSigFigs { input, digits } => compile(input)?.round_sig_figs(*digits),
        ExprNode::Truncate { input, decimals } => compile(input)?.truncate(*decimals),
        ExprNode::Pi => polars::prelude::Expr::pi(),
        ExprNode::Clip { input, min, max } => {
            let input = compile(input)?;
            match (min, max) {
                (Some(min), Some(max)) => input.clip(compile(min)?, compile(max)?),
                (Some(min), None) => input.clip_min(compile(min)?),
                (None, Some(max)) => input.clip_max(compile(max)?),
                (None, None) => {
                    return Err(TerlanPolarsError::new(
                        "invalid_expression",
                        "clip requires at least one boundary",
                    ))
                }
            }
        }
        ExprNode::Entropy {
            input,
            base,
            normalize,
        } => compile(input)?.entropy(*base, *normalize),
        ExprNode::Skew { input, bias } => compile(input)?.skew(*bias),
        ExprNode::Kurtosis {
            input,
            fisher,
            bias,
        } => compile(input)?.kurtosis(*fisher, *bias),
        ExprNode::Between {
            input,
            lower,
            upper,
            closed,
        } => compile(input)?.is_between(
            compile(lower)?,
            compile(upper)?,
            match closed {
                ClosedIntervalKind::Both => ClosedInterval::Both,
                ClosedIntervalKind::Left => ClosedInterval::Left,
                ClosedIntervalKind::Right => ClosedInterval::Right,
                ClosedIntervalKind::None => ClosedInterval::None,
            },
        ),
        ExprNode::IsIn {
            input,
            other,
            nulls_equal,
        } => {
            let input = compile(input)?;
            let other = compile(other)?;
            input
                .clone()
                .is_in(validated_membership_values(other, input), *nulls_equal)
        }
        ExprNode::IsClose {
            input,
            other,
            absolute_tolerance,
            relative_tolerance,
            nans_equal,
        } => compile(input)?.is_close(
            compile(other)?,
            *absolute_tolerance,
            *relative_tolerance,
            *nans_equal,
        ),
        ExprNode::BooleanReduction {
            input,
            kind,
            ignore_nulls,
        } => {
            let input = compile(input)?;
            match kind {
                BooleanReductionKind::Any => input.any(*ignore_nulls),
                BooleanReductionKind::All => input.all(*ignore_nulls),
                BooleanReductionKind::IsEmpty => input.is_empty(*ignore_nulls),
            }
        }
        ExprNode::SplitFirst { input, separator } => compile(input)?
            .str()
            .split(lit(separator.clone()))
            .list()
            .first(),
        ExprNode::StringSplit { input, separator } => {
            compile(input)?.str().split(lit(separator.clone()))
        }
        ExprNode::DateFormat { input, format } => compile(input)?.dt().to_string(format),
        ExprNode::UppercaseNames { input } => compile(input)?.name().to_uppercase(),
        ExprNode::Cast {
            input,
            data_type,
            strict,
        } => {
            let input = compile(input)?;
            let data_type = parse_data_type(data_type)?;
            if *strict {
                polars2_cast_expr(
                    input,
                    data_type,
                    polars::chunked_array::cast::CastOptions::Strict,
                    true,
                )
            } else {
                polars2_cast_expr(
                    input,
                    data_type,
                    polars::chunked_array::cast::CastOptions::NonStrict,
                    false,
                )
            }
        }
        ExprNode::CastWithOptions {
            input,
            data_type,
            mode,
        } => polars2_cast_expr(
            compile(input)?,
            parse_data_type(data_type)?,
            match mode {
                CastMode::Strict => polars::chunked_array::cast::CastOptions::Strict,
                CastMode::NonStrict => polars::chunked_array::cast::CastOptions::NonStrict,
                CastMode::Overflowing => polars::chunked_array::cast::CastOptions::Overflowing,
            },
            matches!(mode, CastMode::Strict),
        ),
        ExprNode::FillNullStrategy {
            input,
            strategy,
            limit,
        } => {
            let strategy = match strategy {
                FillNullStrategyKind::Forward => polars::prelude::FillNullStrategy::Forward(*limit),
                FillNullStrategyKind::Backward => {
                    polars::prelude::FillNullStrategy::Backward(*limit)
                }
                FillNullStrategyKind::Min => polars::prelude::FillNullStrategy::Min,
                FillNullStrategyKind::Max => polars::prelude::FillNullStrategy::Max,
                FillNullStrategyKind::Mean => polars::prelude::FillNullStrategy::Mean,
                FillNullStrategyKind::Zero => polars::prelude::FillNullStrategy::Zero,
                FillNullStrategyKind::One => polars::prelude::FillNullStrategy::One,
            };
            compile(input)?.fill_null_with_strategy(strategy)
        }
        ExprNode::SetSortedFlag {
            input,
            descending,
            nulls_last,
        } => compile(input)?.set_sorted_flag(
            polars_plan::plans::AExprSorted::default()
                .with_desc(*descending)
                .with_nulls_last(*nulls_last),
        ),
        ExprNode::IsSorted {
            input,
            descending,
            nulls_last,
        } => compile(input)?.is_sorted(*descending, *nulls_last),
        ExprNode::ParseDatetime { input } => compile(input)?.str().to_datetime(
            Some(TimeUnit::Microseconds),
            None,
            StrptimeOptions::default(),
            lit("raise"),
        ),
        ExprNode::StringExtract {
            input,
            pattern,
            group,
            all,
        } => {
            let input = compile(input)?.str();
            if *all {
                input.extract_all(lit(pattern.clone()))
            } else {
                input.extract(lit(pattern.clone()), *group)
            }
        }
        ExprNode::StringReplace {
            input,
            pattern,
            replacement,
            all,
        } => {
            let input = compile(input)?.str();
            if *all {
                input.replace_all(lit(pattern.clone()), lit(replacement.clone()), false)
            } else {
                input.replace(lit(pattern.clone()), lit(replacement.clone()), false)
            }
        }
        ExprNode::StringSlice {
            input,
            offset,
            kind,
        } => {
            let input = compile(input)?.str();
            let offset = compile(offset)?;
            match kind {
                StringSliceKind::Slice => input.slice(offset, lit(polars::prelude::NULL)),
                StringSliceKind::Head => input.head(offset),
                StringSliceKind::Tail => input.tail(offset),
            }
        }
        ExprNode::StringFind {
            input,
            pattern,
            literal,
            strict,
        } => {
            let input = compile(input)?.str();
            let pattern = compile(pattern)?;
            if *literal {
                input.find_literal(pattern)
            } else {
                input.find(pattern, *strict)
            }
        }
        ExprNode::StringCountMatches {
            input,
            pattern,
            literal,
        } => compile(input)?
            .str()
            .count_matches(compile(pattern)?, *literal),
        ExprNode::StringPad {
            input,
            length,
            kind,
        } => {
            let input = compile(input)?.str();
            let length = compile(length)?;
            match kind {
                StringPadKind::Start(fill_char) => input.pad_start(length, *fill_char),
                StringPadKind::End(fill_char) => input.pad_end(length, *fill_char),
                StringPadKind::ZFill => input.zfill(length),
            }
        }
        ExprNode::StringDecode {
            input,
            encoding,
            strict,
        } => {
            let input = compile(input)?.str();
            match encoding {
                BinaryEncoding::Hex => input.hex_decode(*strict),
                BinaryEncoding::Base64 => input.base64_decode(*strict),
            }
        }
        ExprNode::StringNormalize { input, form } => {
            use polars::prelude::UnicodeForm;

            compile(input)?.str().normalize(match form {
                StringNormalizationForm::Nfc => UnicodeForm::NFC,
                StringNormalizationForm::Nfkc => UnicodeForm::NFKC,
                StringNormalizationForm::Nfd => UnicodeForm::NFD,
                StringNormalizationForm::Nfkd => UnicodeForm::NFKD,
            })
        }
        ExprNode::Fold {
            kind,
            initial,
            inputs,
            separator,
        } => {
            let inputs = inputs.iter().map(compile).collect::<Result<Vec<_>, _>>()?;
            match kind {
                FoldKind::Sum => inputs.into_iter().fold(
                    compile(initial.as_ref().expect("fold initial"))?,
                    |acc, value| acc + value,
                ),
                FoldKind::Product => inputs.into_iter().fold(
                    compile(initial.as_ref().expect("fold initial"))?,
                    |acc, value| acc * value,
                ),
                FoldKind::HorizontalSum => polars::lazy::dsl::sum_horizontal(inputs, true)?,
                FoldKind::HorizontalAll => polars::lazy::dsl::all_horizontal(inputs)?,
                FoldKind::HorizontalAny => polars::lazy::dsl::any_horizontal(inputs)?,
                FoldKind::ConcatString => {
                    polars::lazy::dsl::concat_str(inputs, separator.as_deref().unwrap_or(""), false)
                }
            }
        }
        ExprNode::FilterExpression { input, predicate } => {
            compile(input)?.filter(compile(predicate)?)
        }
        ExprNode::SliceExpression {
            input,
            offset,
            length,
        } => compile(input)?.slice(compile(offset)?, compile(length)?),
        ExprNode::Append {
            input,
            other,
            upcast,
        } => compile(input)?.append(compile(other)?, *upcast),
        ExprNode::ArgSort {
            input,
            descending,
            nulls_last,
        } => compile(input)?.arg_sort(*descending, *nulls_last),
        ExprNode::SearchSorted {
            input,
            element,
            side,
            descending,
        } => compile(input)?.search_sorted(
            compile(element)?,
            match side {
                SearchSide::Any => polars::prelude::SearchSortedSide::Any,
                SearchSide::Left => polars::prelude::SearchSortedSide::Left,
                SearchSide::Right => polars::prelude::SearchSortedSide::Right,
            },
            *descending,
        ),
        ExprNode::Gather {
            input,
            index,
            scalar,
            null_on_oob,
        } => {
            let input = compile(input)?;
            if *scalar {
                input.get(compile(index)?, *null_on_oob)
            } else {
                input.gather(compile(index)?, *null_on_oob)
            }
        }
        ExprNode::SortOptions {
            input,
            descending,
            nulls_last,
            maintain_order,
        } => {
            // Polars 2.0 removed `maintain_order` from expression sorting and
            // its optimizer requires this internal flag to be false. Equal
            // values are indistinguishable in a single sorted expression, so
            // retain the option in the portable descriptor while compiling
            // through the supported 2.0 expression API.
            let _ = maintain_order;
            compile(input)?.sort(polars::prelude::SortOptions {
                descending: *descending,
                nulls_last: *nulls_last,
                maintain_order: false,
                ..Default::default()
            })
        }
        ExprNode::ShiftAndFill {
            input,
            periods,
            fill,
        } => compile(input)?.shift_and_fill(compile(periods)?, compile(fill)?),
        ExprNode::Diff {
            input,
            periods,
            behavior,
        } => compile(input)?.diff(
            compile(periods)?,
            match behavior {
                DiffBehavior::Ignore => polars::series::ops::NullBehavior::Ignore,
                DiffBehavior::Drop => polars::series::ops::NullBehavior::Drop,
            },
        ),
        ExprNode::GatherEvery {
            input,
            step,
            offset,
        } => compile(input)?.gather_every(*step, *offset),
        ExprNode::ExtendConstant {
            input,
            value,
            count,
        } => compile(input)?.extend_constant(compile(value)?, compile(count)?),
        ExprNode::SortBy {
            input,
            by,
            descending,
        } => compile(input)?.sort_by(
            by.iter().map(compile).collect::<Result<Vec<_>, _>>()?,
            polars::prelude::SortMultipleOptions::default().with_order_descending(*descending),
        ),
        ExprNode::SortByOptions {
            input,
            by,
            descending,
            nulls_last,
            maintain_order,
        } => compile(input)?.sort_by(
            by.iter().map(compile).collect::<Result<Vec<_>, _>>()?,
            polars::prelude::SortMultipleOptions::default()
                .with_order_descending_multi(descending.clone())
                .with_nulls_last_multi(nulls_last.clone())
                .with_maintain_order(*maintain_order),
        ),
        ExprNode::Over {
            input,
            keys,
            mapping,
        } => {
            let input = compile(input)?;
            let keys = keys.iter().map(compile).collect::<Result<Vec<_>, _>>()?;
            let mapping = match mapping {
                WindowMappingKind::GroupsToRows => polars::prelude::WindowMapping::GroupsToRows,
                WindowMappingKind::Explode => polars::prelude::WindowMapping::Explode,
                WindowMappingKind::Join => polars::prelude::WindowMapping::Join,
            };
            input.over_with_options(Some(keys), None, mapping)?
        }
        ExprNode::OverOrdered {
            input,
            partition_by,
            order_by,
            descending,
            nulls_last,
            mapping,
        } => {
            let partition_by = if partition_by.is_empty() {
                None
            } else {
                Some(
                    partition_by
                        .iter()
                        .map(compile)
                        .collect::<Result<Vec<_>, _>>()?,
                )
            };
            let order_by = order_by
                .iter()
                .map(compile)
                .collect::<Result<Vec<_>, _>>()?;
            let options = polars::prelude::SortOptions::default()
                .with_order_descending(*descending)
                .with_nulls_last(*nulls_last);
            let mapping = match mapping {
                WindowMappingKind::GroupsToRows => polars::prelude::WindowMapping::GroupsToRows,
                WindowMappingKind::Explode => polars::prelude::WindowMapping::Explode,
                WindowMappingKind::Join => polars::prelude::WindowMapping::Join,
            };
            compile(input)?.over_with_options(partition_by, Some((order_by, options)), mapping)?
        }
        ExprNode::HeadExpression { input, limit } => compile(input)?.head(Some(*limit)),
        ExprNode::TailExpression { input, limit } => compile(input)?.tail(Some(*limit)),
        ExprNode::Struct { inputs } => {
            polars::lazy::dsl::as_struct(inputs.iter().map(compile).collect::<Result<Vec<_>, _>>()?)
        }
        ExprNode::StructField { input, name } => compile(input)?.struct_().field_by_name(name),
        ExprNode::StructFieldAt { input, index } => {
            compile(input)?.struct_().field_by_index(*index)
        }
        ExprNode::StructFields { input, names } => {
            compile(input)?.struct_().field_by_names(names.clone())
        }
        ExprNode::StructRenameFields { input, names } => {
            require_struct_width(compile(input)?, names.len())
                .struct_()
                .rename_fields(names.clone())
        }
        ExprNode::StructWithFields { input, fields } => compile(input)?
            .struct_()
            .with_fields(fields.iter().map(compile).collect::<Result<Vec<_>, _>>()?),
        ExprNode::Conditional {
            predicate,
            truthy,
            falsy,
        } => when(compile(predicate)?)
            .then(compile(truthy)?)
            .otherwise(compile(falsy)?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expressions_round_trip_without_polars() {
        let column = expr_col("height").unwrap();
        let factor = expr_lit(&TerlanPolarsScalar::Float(0.95)).unwrap();
        let scaled = expr_multiply(&column, &factor).unwrap();
        let rounded = expr_round(&scaled, 2).unwrap();
        let renamed = expr_suffix(&rounded, "-5%").unwrap();
        assert!(decode(&renamed).is_ok());
    }

    #[test]
    fn expression_tree_traversal_and_rewriting_are_structural() {
        let column = expr_col("value").unwrap();
        let literal = expr_lit(&TerlanPolarsScalar::Int(1)).unwrap();
        let sum = expr_add(&column, &literal).unwrap();

        let nodes = expr_meta_nodes(&sum).unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0], sum);
        assert_eq!(expr_meta_children(&sum).unwrap(), [column, literal.clone()]);

        let parameter = crate::expression_udf_parameter(0).unwrap();
        let udf = crate::define_expression_udf(1, "Int64", &parameter).unwrap();
        let children_mapped = expr_map_children(&sum, &udf).unwrap();
        let mapped_value = expression_value(&children_mapped).unwrap();
        assert_eq!(mapped_value["op"], "binary");
        assert_eq!(mapped_value["left"]["op"], "cast");
        assert_eq!(mapped_value["right"]["op"], "cast");

        let rewritten = expr_rewrite(&sum, &udf).unwrap();
        let rewritten_value = expression_value(&rewritten).unwrap();
        assert_eq!(rewritten_value["op"], "cast");
        assert_eq!(rewritten_value["input"]["op"], "binary");

        let zero_arity = crate::define_expression_udf(0, "Int64", &literal).unwrap();
        assert_eq!(
            expr_rewrite(&sum, &zero_arity).unwrap_err().code(),
            "udf_arity_mismatch"
        );
    }

    #[test]
    fn constant_integer_extraction_matches_literal_semantics() {
        let seven = expr_lit(&TerlanPolarsScalar::Int(7)).unwrap();
        let two = expr_lit(&TerlanPolarsScalar::Int(2)).unwrap();
        let five = expr_subtract(&seven, &two).unwrap();
        assert_eq!(expr_extract_i64(&five).unwrap(), 5);
        assert_eq!(expr_extract_usize(&five).unwrap(), 5);

        let negative = expr_subtract(&two, &seven).unwrap();
        assert!(expr_extract_usize(&negative).is_err());
        assert!(expr_extract_i64(&expr_col("value").unwrap()).is_err());
    }

    #[test]
    fn option_bearing_expression_builders_reject_invalid_options() {
        let value = expr_col("value").unwrap();
        let key = expr_col("key").unwrap();

        assert!(expr_cast_with_options(&value, "Int64", "lossy").is_err());
        assert!(expr_fill_null_with_strategy(&value, "median", -1).is_err());
        assert!(expr_fill_null_with_strategy(&value, "zero", 1).is_err());
        assert!(expr_sort_by(&value, &[], &[false], &[false], false).is_err());
        assert!(expr_sort_by(&value, std::slice::from_ref(&key), &[], &[false], false).is_err());
        assert!(expr_over_with_options(&value, &[], &[], false, false, "groups_to_rows").is_err());

        let arithmetic = expr_add(&value, &key).unwrap();
        assert!(expr_into_selector(&arithmetic).is_err());
        assert!(expr_into_selector(&value).is_ok());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn option_bearing_expression_operations_execute_in_polars() {
        use polars::prelude::*;

        let two = expr_lit(&TerlanPolarsScalar::Int(2)).unwrap();
        let true_div = expr_alias(
            &expr_true_divide(&expr_col("integer").unwrap(), &two).unwrap(),
            "true_div",
        )
        .unwrap();
        let overflowing = expr_alias(
            &expr_cast_with_options(&expr_col("integer").unwrap(), "Int8", "overflowing").unwrap(),
            "overflowing",
        )
        .unwrap();
        let parsed = expr_alias(
            &expr_cast_with_options(&expr_col("text").unwrap(), "Int64", "non_strict").unwrap(),
            "parsed",
        )
        .unwrap();
        let result = df!("integer" => [5_i64, 300], "text" => ["7", "bad"])
            .unwrap()
            .lazy()
            .select([
                compile_one(&true_div).unwrap(),
                compile_one(&overflowing).unwrap(),
                compile_one(&parsed).unwrap(),
            ])
            .collect()
            .unwrap();
        assert_eq!(
            result.column("true_div").unwrap().f64().unwrap().get(0),
            Some(2.5)
        );
        assert_eq!(
            result.column("overflowing").unwrap().i8().unwrap().get(1),
            Some(44)
        );
        assert_eq!(
            result.column("parsed").unwrap().i64().unwrap().get(0),
            Some(7)
        );
        assert_eq!(result.column("parsed").unwrap().i64().unwrap().get(1), None);

        let forward = expr_alias(
            &expr_fill_null_with_strategy(&expr_col("nullable").unwrap(), "forward", 1).unwrap(),
            "forward",
        )
        .unwrap();
        let zero = expr_alias(
            &expr_fill_null_with_strategy(&expr_col("nullable").unwrap(), "zero", -1).unwrap(),
            "zero",
        )
        .unwrap();
        let filled = df!("nullable" => &[Some(1_i64), None, None, Some(4)])
            .unwrap()
            .lazy()
            .select([compile_one(&forward).unwrap(), compile_one(&zero).unwrap()])
            .collect()
            .unwrap();
        let forward_values = filled.column("forward").unwrap().i64().unwrap();
        assert_eq!(
            (0..forward_values.len())
                .map(|index| forward_values.get(index))
                .collect::<Vec<_>>(),
            [Some(1), Some(1), None, Some(4)]
        );
        assert_eq!(
            filled
                .column("zero")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            [1, 0, 0, 4]
        );

        let selector = expr_into_selector(&expr_col("value").unwrap()).unwrap();
        let sorted = expr_alias(
            &expr_sort_by(
                &selector,
                &[expr_col("key").unwrap()],
                &[false],
                &[true],
                true,
            )
            .unwrap(),
            "sorted",
        )
        .unwrap();
        let sorted_frame = df!(
            "key" => &[Some(2_i64), None, Some(1)],
            "value" => ["b", "n", "a"]
        )
        .unwrap()
        .lazy()
        .select([compile_one(&sorted).unwrap()])
        .collect()
        .unwrap();
        let sorted_values = sorted_frame.column("sorted").unwrap().str().unwrap();
        assert_eq!(
            (0..sorted_values.len())
                .map(|index| sorted_values.get(index).unwrap())
                .collect::<Vec<_>>(),
            ["a", "b", "n"]
        );

        let cumulative = expr_cum_sum(&expr_col("value").unwrap(), false).unwrap();
        let windowed = expr_alias(
            &expr_over_with_options(
                &cumulative,
                &[expr_col("group").unwrap()],
                &[expr_col("order").unwrap()],
                false,
                false,
                "groups_to_rows",
            )
            .unwrap(),
            "windowed",
        )
        .unwrap();
        let window_result = df!(
            "group" => ["x", "x", "x"],
            "order" => [2_i64, 1, 3],
            "value" => [20_i64, 10, 30]
        )
        .unwrap()
        .lazy()
        .select([compile_one(&windowed).unwrap()])
        .collect()
        .unwrap();
        assert_eq!(
            window_result
                .column("windowed")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            [30, 10, 60]
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn expression_tree_rewrites_execute_in_polars() {
        use polars::prelude::*;

        let parameter = crate::expression_udf_parameter(0).unwrap();
        let udf = crate::define_expression_udf(1, "Int64", &parameter).unwrap();
        let expression = expr_add(
            &expr_col("value").unwrap(),
            &expr_lit(&TerlanPolarsScalar::Int(1)).unwrap(),
        )
        .unwrap();
        let mapped = expr_alias(&expr_map_children(&expression, &udf).unwrap(), "mapped").unwrap();
        let rewritten = expr_alias(&expr_rewrite(&expression, &udf).unwrap(), "rewritten").unwrap();
        let result = df!("value" => [1_i64, 2, 3])
            .unwrap()
            .lazy()
            .select([
                compile_one(&mapped).unwrap(),
                compile_one(&rewritten).unwrap(),
            ])
            .collect()
            .unwrap();
        assert_eq!(
            result
                .column("mapped")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            [2, 3, 4]
        );
        assert_eq!(
            result
                .column("rewritten")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            [2, 3, 4]
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn direct_numeric_and_physical_operations_execute_in_polars() {
        use polars::prelude::*;

        let value = expr_col("value").unwrap();
        let result = df!("value" => [8.0_f64, 27.0])
            .unwrap()
            .lazy()
            .select([
                compile_one(&expr_alias(&expr_neg(&value).unwrap(), "neg").unwrap()).unwrap(),
                compile_one(&expr_alias(&expr_sqrt(&value).unwrap(), "sqrt").unwrap()).unwrap(),
                compile_one(&expr_alias(&expr_cbrt(&value).unwrap(), "cbrt").unwrap()).unwrap(),
            ])
            .collect()
            .unwrap();
        assert_eq!(
            result.column("neg").unwrap().f64().unwrap().get(0),
            Some(-8.0)
        );
        assert_eq!(
            result.column("sqrt").unwrap().f64().unwrap().get(0),
            Some(8.0_f64.sqrt())
        );
        assert_eq!(
            result.column("cbrt").unwrap().f64().unwrap().get(1),
            Some(3.0)
        );

        let sorted =
            expr_set_sorted_flag(&expr_col("value").unwrap(), "ascending", "last").unwrap();
        let sorted_result = df!("value" => [1_i64, 2, 3])
            .unwrap()
            .lazy()
            .select([compile_one(&sorted).unwrap()])
            .collect()
            .unwrap();
        assert_eq!(
            sorted_result
                .column("value")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );

        let date = expr_strict_cast(&expr_col("value").unwrap(), "Date").unwrap();
        let physical = expr_alias(&expr_to_physical(&date).unwrap(), "physical").unwrap();
        let physical_result = df!("value" => [0_i32, 1])
            .unwrap()
            .lazy()
            .select([compile_one(&physical).unwrap()])
            .collect()
            .unwrap();
        assert_eq!(
            physical_result.column("physical").unwrap().dtype(),
            &DataType::Int32
        );

        assert!(expr_set_sorted_flag(&value, "sideways", "last").is_err());
        assert!(expr_set_sorted_flag(&value, "ascending", "middle").is_err());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn temporal_rolling_group_expression_executes_in_polars() {
        use polars::prelude::*;

        let aggregation = expr_sum(&expr_col("value").unwrap()).unwrap();
        let rolling = expr_alias(
            &expr_rolling(
                &aggregation,
                &expr_col("index").unwrap(),
                "2i",
                "-2i",
                "right",
            )
            .unwrap(),
            "rolling",
        )
        .unwrap();
        let result = df!("index" => [0_i64, 1, 2], "value" => [1_i64, 2, 3])
            .unwrap()
            .lazy()
            .select([compile_one(&rolling).unwrap()])
            .collect()
            .unwrap();
        assert_eq!(
            result
                .column("rolling")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            [1, 3, 5]
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn expression_to_field_resolves_name_and_data_type() {
        let expression = expr_add(
            &expr_col("value").unwrap(),
            &expr_lit(&TerlanPolarsScalar::Int(1)).unwrap(),
        )
        .unwrap();
        let field =
            expr_to_field(&expression, &["value".to_string()], &["Int64".to_string()]).unwrap();
        assert_eq!(
            serde_json::from_str::<DataTypeDescriptor>(&field).unwrap(),
            DataTypeDescriptor::Field {
                name: "value".to_string(),
                data_type: "Int64".to_string(),
            }
        );
        assert!(expr_to_field(&expression, &["value".to_string()], &[]).is_err());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn declarative_name_and_struct_field_transforms_execute_in_polars() {
        use polars::prelude::*;

        let renamed = expr_map_name(&expr_col("Value").unwrap(), "lowercase", "", "").unwrap();
        let structure =
            expr_as_struct(&[expr_col("first").unwrap(), expr_col("second").unwrap()]).unwrap();
        let mapped_fields = expr_map_field_names(&structure, "uppercase", "", "").unwrap();
        let selected_field = expr_struct_field(&mapped_fields, "FIRST").unwrap();
        let result = df!(
            "Value" => [1_i64],
            "first" => [2_i64],
            "second" => [3_i64]
        )
        .unwrap()
        .lazy()
        .select([
            compile_one(&renamed).unwrap(),
            compile_one(&selected_field).unwrap(),
        ])
        .collect()
        .unwrap();
        assert_eq!(
            result
                .get_column_names()
                .iter()
                .map(|name| name.as_str())
                .collect::<Vec<_>>(),
            ["value", "FIRST"]
        );
        assert_eq!(
            result.column("FIRST").unwrap().i64().unwrap().get(0),
            Some(2)
        );

        assert!(expr_map_name(&expr_col("Value").unwrap(), "unknown", "", "").is_err());
        assert!(expr_map_field_names(&structure, "replace", "", "x").is_err());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn cheat_sheet_range_list_and_index_expressions_execute() {
        use polars::prelude::*;

        let start = expr_lit(&TerlanPolarsScalar::Int(1)).unwrap();
        let end = expr_lit(&TerlanPolarsScalar::Int(5)).unwrap();
        let range =
            expr_alias(&expr_int_range(&start, &end, 1, "Int64").unwrap(), "range").unwrap();
        let range_frame = DataFrame::empty()
            .lazy()
            .select([compile_one(&range).unwrap()])
            .collect()
            .unwrap();
        assert_eq!(
            range_frame.column("range").unwrap().i64().unwrap().get(3),
            Some(4)
        );

        let frame = df!("a" => [-2_i64, 0, 3], "b" => [10_i64, 20, 30]).unwrap();
        let a = expr_col("a").unwrap();
        let b = expr_col("b").unwrap();
        let list = expr_alias(&expr_concat_list(&[a.clone(), b]).unwrap(), "items").unwrap();
        let sign = expr_alias(&expr_sign(&a).unwrap(), "sign").unwrap();
        let truth = expr_gt(&a, &expr_lit(&TerlanPolarsScalar::Int(0)).unwrap()).unwrap();
        let indices = expr_alias(&expr_arg_true(&truth).unwrap(), "indices").unwrap();
        let selected = frame
            .clone()
            .lazy()
            .select([compile_one(&list).unwrap(), compile_one(&sign).unwrap()])
            .collect()
            .unwrap();
        let selected_indices = frame
            .lazy()
            .select([compile_one(&indices).unwrap()])
            .collect()
            .unwrap();
        assert_eq!(selected.column("items").unwrap().len(), 3);
        assert_eq!(
            selected.column("sign").unwrap().i64().unwrap().get(0),
            Some(-1)
        );
        assert_eq!(
            selected_indices
                .column("indices")
                .unwrap()
                .u32()
                .unwrap()
                .get(0),
            Some(2)
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn cheat_sheet_temporal_constructors_execute() {
        use polars::prelude::*;

        let start = expr_date("2026-01-01").unwrap();
        let end = expr_date("2026-01-03").unwrap();
        let dates = expr_alias(
            &expr_date_range(&start, &end, "1d", "both").unwrap(),
            "dates",
        )
        .unwrap();
        let date_frame = DataFrame::empty()
            .lazy()
            .select([compile_one(&dates).unwrap()])
            .collect()
            .unwrap();
        assert_eq!(date_frame.column("dates").unwrap().len(), 3);

        let zero = expr_lit(&TerlanPolarsScalar::Int(0)).unwrap();
        let one = expr_lit(&TerlanPolarsScalar::Int(1)).unwrap();
        let duration = expr_alias(
            &expr_duration(
                &[
                    zero.clone(),
                    one,
                    zero.clone(),
                    zero.clone(),
                    zero.clone(),
                    zero.clone(),
                    zero.clone(),
                    zero,
                ],
                "us",
            )
            .unwrap(),
            "duration",
        )
        .unwrap();
        let duration_frame = DataFrame::empty()
            .lazy()
            .select([compile_one(&duration).unwrap()])
            .collect()
            .unwrap();
        assert_eq!(duration_frame.column("duration").unwrap().len(), 1);
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn cheat_sheet_selector_algebra_executes() {
        use polars::prelude::*;

        let frame = df!(
            "name" => ["Ada", "Grace"],
            "age" => [36_i64, 85],
            "city" => ["London", "Arlington"]
        )
        .unwrap();
        let selected = expr_selector_union(
            &expr_selector_numeric().unwrap(),
            &expr_selector_starts_with("na").unwrap(),
        )
        .unwrap();
        let result = frame
            .clone()
            .lazy()
            .select([compile_one(&selected).unwrap()])
            .collect()
            .unwrap();
        assert_eq!(result.width(), 2);
        assert!(result.column("name").is_ok());
        assert!(result.column("age").is_ok());

        let strings_without_city = expr_selector_difference(
            &expr_selector_string().unwrap(),
            &expr_selector_names(&["city".to_string()]).unwrap(),
        )
        .unwrap();
        let result = frame
            .lazy()
            .select([compile_one(&strings_without_city).unwrap()])
            .collect()
            .unwrap();
        assert_eq!(result.width(), 1);
        assert!(result.column("name").is_ok());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn cheat_sheet_rolling_map_executes_declarative_udf_per_window() {
        use polars::prelude::*;

        let parameter = crate::expression_udf_parameter(0).unwrap();
        let template = expr_mean(&parameter).unwrap();
        let udf = crate::define_expression_udf(1, "float64", &template).unwrap();
        let rolling = expr_alias(
            &expr_rolling_map(&expr_col("value").unwrap(), &udf, 2, 2, false).unwrap(),
            "rolling",
        )
        .unwrap();
        let result = df!("value" => [1.0_f64, 3.0, 5.0])
            .unwrap()
            .lazy()
            .select([compile_one(&rolling).unwrap()])
            .collect()
            .unwrap();
        let values = result.column("rolling").unwrap().f64().unwrap();
        assert_eq!(values.get(0), None);
        assert_eq!(values.get(1), Some(2.0));
        assert_eq!(values.get(2), Some(4.0));
    }
}
