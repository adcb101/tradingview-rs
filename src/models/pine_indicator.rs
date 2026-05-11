use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use ustr::Ustr;

use crate::{
    Result,
    chart::study::{IndicatorInput, InputValue},
    client::misc::get_indicator_metadata,
    models::{FinancialPeriod, UserCookies},
};

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinIndicators {
    All,
    Fundamental,
    Standard,
    Candlestick,
}

impl std::fmt::Display for BuiltinIndicators {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BuiltinIndicators::All => write!(f, "all"),
            BuiltinIndicators::Fundamental => write!(f, "fundamental"),
            BuiltinIndicators::Standard => write!(f, "standard"),
            BuiltinIndicators::Candlestick => write!(f, "candlestick"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PineInfo {
    pub user_id: i64,
    pub script_name: String,
    pub script_source: String,
    #[serde(rename(deserialize = "scriptIdPart"))]
    pub script_id: String,
    pub script_access: String,
    #[serde(rename(deserialize = "version"))]
    pub script_version: String,
    pub extra: PineInfoExtra,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PineInfoExtra {
    pub financial_period: Option<FinancialPeriod>,
    pub fund_id: Option<String>,
    pub fundamental_category: Option<String>,
    pub is_auto: bool,
    pub is_beta: bool,
    pub is_built_in: bool,
    pub is_candle_stick: bool,
    pub is_fundamental_study: bool,
    pub is_hidden_study: bool,
    pub is_mtf_resolution: bool,
    pub is_new: bool,
    pub is_pine_editor_new_template: bool,
    pub is_updated: bool,
    pub kind: String,
    pub short_description: String,
    pub source_inputs_count: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TranslateResponse {
    pub success: bool,
    pub result: PineMetadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub next: String,
    pub results: Vec<PineSearchResult>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PineSearchResult {
    pub image_url: String,
    pub script_name: String,
    pub script_source: String,
    pub access: i64,
    pub script_id_part: String,
    pub version: String,
    pub extra: PineSearchExtra,
    pub agree_count: i64,
    pub author: PineSearchAuthor,
    pub weight: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PineSearchExtra {
    pub kind: Option<String>,
    pub source_inputs_count: Option<i64>,
    #[serde(rename = "isMTFResolution")]
    pub is_mtf_resolution: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PineSearchAuthor {
    pub id: i64,
    pub username: Ustr,
    #[serde(rename = "is_broker")]
    pub is_broker: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct PineMetadata {
    #[serde(rename(deserialize = "IL"))]
    pub il: Ustr,
    #[serde(rename(deserialize = "ilTemplate"))]
    pub il_template: Ustr,
    #[serde(rename(deserialize = "metaInfo"))]
    pub data: PineMetadataInfo,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct PineMetadataInfo {
    pub id: Ustr,
    #[serde(rename(deserialize = "scriptIdPart"))]
    pub script_id: Ustr,
    pub description: Ustr,
    pub short_description: Ustr,
    pub financial_period: Option<FinancialPeriod>,
    pub grouping_key: Ustr,
    pub is_fundamental_study: bool,
    pub is_hidden_study: bool,
    pub is_tv_script: bool,
    pub is_tv_script_stub: bool,
    pub is_price_study: bool,
    pub inputs: Vec<PineInput>,
    pub defaults: HashMap<String, Value>,
    pub palettes: HashMap<String, Value>,
    pub pine: HashMap<String, String>,
    pub plots: Vec<Plot>,
    pub styles: HashMap<String, Value>,
    pub uses_private_lib: bool,
    pub warnings: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Plot {
    pub id: String,
    pub plot_type: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct PineInput {
    pub name: String,
    pub inline: String,
    pub id: String,
    pub defval: Value,
    pub is_hidden: bool,
    pub is_fake: bool,
    pub optional: bool,
    pub options: Vec<String>,
    pub tooltip: Option<String>,
    #[serde(rename(deserialize = "type"))]
    pub input_type: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Copy)]
pub enum ScriptType {
    #[default]
    Script,
    IntervalScript,
    StrategyScript,
    VolumeBasicStudies,
    FixedBasicStudies,
    FixedVolumeByPrice,
    SessionVolumeByPrice,
    SessionRoughVolumeByPrice,
    SessionDetailedVolumeByPrice,
    VisibleVolumeByPrice,
}

impl std::fmt::Display for ScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScriptType::Script => write!(f, "Script@tv-scripting-101!"),
            ScriptType::IntervalScript => write!(f, "Internal@tv-scripting-101!"),
            ScriptType::StrategyScript => write!(f, "StrategyScript@tv-scripting-101!"),
            ScriptType::VolumeBasicStudies => write!(f, "Volume@tv-basicstudies-241"),
            ScriptType::FixedBasicStudies => write!(f, "VbPFixed@tv-basicstudies-241"),
            ScriptType::FixedVolumeByPrice => write!(f, "VbPFixed@tv-volumebyprice-53!"),
            ScriptType::SessionVolumeByPrice => write!(f, "VbPSessions@tv-volumebyprice-53"),
            ScriptType::SessionRoughVolumeByPrice => {
                write!(f, "VbPSessionsRough@tv-volumebyprice-53!")
            }
            ScriptType::SessionDetailedVolumeByPrice => {
                write!(f, "VbPSessionsDetailed@tv-volumebyprice-53!")
            }
            ScriptType::VisibleVolumeByPrice => write!(f, "VbPVisible@tv-volumebyprice-53"),
        }
    }
}

impl From<String> for ScriptType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Script@tv-scripting-101!" => ScriptType::Script,
            "Internal@tv-scripting-101!" => ScriptType::IntervalScript,
            "StrategyScript@tv-scripting-101!" => ScriptType::StrategyScript,
            "Volume@tv-basicstudies-241" | "Volume@tv-basicstudies-144" => {
                ScriptType::VolumeBasicStudies
            }
            "VbPFixed@tv-basicstudies-241" | "VbPFixed@tv-basicstudies-241!" | "VbPFixed@tv-basicstudies-139!" => {
                ScriptType::FixedBasicStudies
            }
            "VbPFixed@tv-volumebyprice-53!" => ScriptType::FixedVolumeByPrice,
            "VbPSessions@tv-volumebyprice-53" => ScriptType::SessionVolumeByPrice,
            "VbPSessionsRough@tv-volumebyprice-53!" => ScriptType::SessionRoughVolumeByPrice,
            "VbPSessionsDetailed@tv-volumebyprice-53!" => ScriptType::SessionDetailedVolumeByPrice,
            "VbPVisible@tv-volumebyprice-53" => ScriptType::VisibleVolumeByPrice,
            _ => ScriptType::Script,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PineIndicator {
    pub script_id: Ustr,
    pub script_version: Ustr,
    pub script_type: ScriptType,
    pub metadata: PineMetadata,
}

pub struct PineIndicatorBuilder {
    user: Option<UserCookies>,
}

impl PineIndicatorBuilder {
    pub fn user(&mut self, user: UserCookies) -> &mut Self {
        self.user = Some(user);
        self
    }

    pub async fn fetch(
        &mut self,
        script_id: &str,
        script_version: &str,
        script_type: ScriptType,
    ) -> Result<PineIndicator> {
        let metadata = match &self.user {
            Some(user) => get_indicator_metadata(Some(user), script_id, script_version).await?,
            None => get_indicator_metadata(None, script_id, script_version).await?,
        };
        // Use metadata values (matching TS behavior) rather than raw input params
        let actual_id = if metadata.data.script_id.is_empty() {
            Ustr::from(script_id)
        } else {
            metadata.data.script_id
        };
        let actual_version = metadata
            .data
            .pine
            .get("version")
            .map(|v| Ustr::from(v.as_str()))
            .unwrap_or_else(|| Ustr::from(script_version));
        // Auto-detect script type from metadata if not explicitly set
        let script_type = if metadata.data.is_tv_script {
            script_type
        } else if metadata.data.is_tv_script_stub {
            ScriptType::StrategyScript
        } else {
            script_type
        };
        Ok(PineIndicator {
            script_id: actual_id,
            script_version: actual_version,
            script_type,
            metadata,
        })
    }
}

impl PineIndicator {
    pub fn build() -> PineIndicatorBuilder {
        PineIndicatorBuilder { user: None }
    }

    pub fn to_study_inputs(&self) -> Result<Value> {
        let mut inputs: HashMap<Ustr, IndicatorInput> = HashMap::new();
        inputs.insert(
            Ustr::from("text"),
            IndicatorInput::String(Ustr::from(&self.metadata.il_template)),
        );
        inputs.insert(
            Ustr::from("pineId"),
            IndicatorInput::String(Ustr::from(&self.script_id)),
        );
        inputs.insert(
            Ustr::from("pineVersion"),
            IndicatorInput::String(Ustr::from(&self.script_version)),
        );
        self.metadata.data.inputs.iter().enumerate().for_each(|(n, input)| {
            if input.id == "text" || input.id == "pineId" || input.id == "pineVersion" {
                return;
            }
            // For color inputs, use the index as value (matching TS behavior)
            let v = if input.input_type == "color" {
                Value::from(n)
            } else {
                input.defval.clone()
            };
            inputs.insert(
                Ustr::from(&input.id),
                IndicatorInput::IndicatorInput(InputValue {
                    v,
                    f: Value::from(input.is_fake),
                    t: Value::from(input.input_type.clone()),
                }),
            );
        });

        let json_value = serde_json::to_value(inputs)?;
        Ok(json_value)
    }

    /// Sets an input value on the indicator.
    ///
    /// The `key` can be the input ID (`in_123`), the inline name,
    /// or the internal ID of the input parameter.
    ///
    /// # Arguments
    /// * `key` - The input identifier
    /// * `value` - The new value to set (as JSON Value for type flexibility)
    pub fn set_option(&mut self, key: &str, value: Value) -> Result<()> {
        let inputs = &mut self.metadata.data.inputs;

        // Find the matching input
        let input_idx = inputs.iter().position(|inp| {
            inp.id == key
                || format!("in_{}", inp.id) == key
                || inp.inline == key
                || (inp.id == format!("in_{}", key))
        });

        let Some(idx) = input_idx else {
            return Err(crate::error::Error::IndicatorDataNotFound(Ustr::from(
                &format!("Input '{key}' not found"),
            )));
        };

        let input = &inputs[idx];
        let input_type = input.input_type.as_str();

        // Type checking
        match input_type {
            "bool" => {
                if !value.is_boolean() {
                    return Err(crate::error::Error::TypeConversion(Ustr::from(
                        &format!("Input '{}' ({}) must be a Boolean", input.name, input.id),
                    )));
                }
            }
            "integer" | "float" => {
                if !value.is_number() {
                    return Err(crate::error::Error::TypeConversion(Ustr::from(
                        &format!("Input '{}' ({}) must be a Number", input.name, input.id),
                    )));
                }
            }
            "text" | "resolution" => {
                if !value.is_string() {
                    return Err(crate::error::Error::TypeConversion(Ustr::from(
                        &format!("Input '{}' ({}) must be a String", input.name, input.id),
                    )));
                }
            }
            _ => {}
        }

        // Options validation
        if !input.options.is_empty() {
            let val_str = match &value {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            if !input.options.iter().any(|o| o == &val_str) {
                return Err(crate::error::Error::TypeConversion(Ustr::from(
                    &format!(
                        "Input '{}' ({}) must be one of: {:?}",
                        input.name, input.id, input.options
                    ),
                )));
            }
        }

        inputs[idx].defval = value;
        Ok(())
    }
}

/// A built-in TradingView indicator (Volume Profile variants, etc.)
///
/// These are indicators that come pre-installed with TradingView,
/// as opposed to community Pine Script indicators.
#[derive(Debug, Clone)]
pub struct BuiltInIndicator {
    indicator_type: ScriptType,
    options: HashMap<String, Value>,
}

impl BuiltInIndicator {
    /// Creates a new `BuiltInIndicator` of the given type.
    ///
    /// # Arguments
    /// * `indicator_type` - The built-in indicator type string
    pub fn new(indicator_type: ScriptType) -> Self {
        let options = default_options(indicator_type);
        Self {
            indicator_type,
            options,
        }
    }

    /// Returns the indicator's script type string (for sending to TV).
    pub fn script_type(&self) -> ScriptType {
        self.indicator_type
    }

    /// Returns the current options (for study input construction).
    pub fn options(&self) -> &HashMap<String, Value> {
        &self.options
    }

    /// Sets an option value with type validation.
    ///
    /// By default, validates the value type against the defaults for this indicator type.
    /// Pass `force = true` to skip validation.
    ///
    /// # Arguments
    /// * `key` - The option key name
    /// * `value` - The new value
    /// * `force` - If true, skips type/key validation
    pub fn set_option<V: Into<Value>>(&mut self, key: &str, value: V, force: bool) -> Result<()> {
        let val = value.into();
        if !force {
            let defaults = default_options(self.indicator_type);
            if let Some(default_val) = defaults.get(key) {
                // Check type matches the default
                if default_val.is_boolean() != val.is_boolean()
                    || default_val.is_number() != val.is_number()
                    || default_val.is_string() != val.is_string()
                {
                    return Err(crate::error::Error::TypeConversion(Ustr::from(
                        &format!("Wrong '{key}' value type (must match default type)"),
                    )));
                }
            } else {
                return Err(crate::error::Error::TypeConversion(Ustr::from(
                    &format!("Option '{key}' is not valid for this indicator type"),
                )));
            }
        }
        self.options.insert(key.to_string(), val);
        Ok(())
    }
}

fn default_options(indicator_type: ScriptType) -> HashMap<String, Value> {
    use serde_json::json;

    match indicator_type {
        ScriptType::VolumeBasicStudies => {
            let mut m = HashMap::new();
            m.insert("length".into(), json!(20));
            m.insert("col_prev_close".into(), json!(false));
            m
        }
        ScriptType::FixedBasicStudies
        | ScriptType::FixedVolumeByPrice => {
            let mut m = HashMap::new();
            m.insert("rowsLayout".into(), json!("Number Of Rows"));
            m.insert("rows".into(), json!(24));
            m.insert("volume".into(), json!("Up/Down"));
            m.insert("vaVolume".into(), json!(70));
            m.insert("subscribeRealtime".into(), json!(false));
            m.insert("first_bar_time".into(), Value::Null);
            m.insert("last_bar_time".into(), json!(chrono::Utc::now().timestamp_millis()));
            m.insert("extendToRight".into(), json!(false));
            m.insert("mapRightBoundaryToBarStartTime".into(), json!(true));
            m
        }
        ScriptType::SessionVolumeByPrice => {
            let mut m = HashMap::new();
            m.insert("rowsLayout".into(), json!("Number Of Rows"));
            m.insert("rows".into(), json!(24));
            m.insert("volume".into(), json!("Up/Down"));
            m.insert("vaVolume".into(), json!(70));
            m.insert("extendPocRight".into(), json!(false));
            m
        }
        ScriptType::SessionRoughVolumeByPrice => {
            let mut m = HashMap::new();
            m.insert("volume".into(), json!("Up/Down"));
            m.insert("vaVolume".into(), json!(70));
            m
        }
        ScriptType::SessionDetailedVolumeByPrice => {
            let mut m = HashMap::new();
            m.insert("volume".into(), json!("Up/Down"));
            m.insert("vaVolume".into(), json!(70));
            m.insert("subscribeRealtime".into(), json!(false));
            m.insert("first_visible_bar_time".into(), Value::Null);
            m.insert("last_visible_bar_time".into(), json!(chrono::Utc::now().timestamp_millis()));
            m
        }
        ScriptType::VisibleVolumeByPrice => {
            let mut m = HashMap::new();
            m.insert("rowsLayout".into(), json!("Number Of Rows"));
            m.insert("rows".into(), json!(24));
            m.insert("volume".into(), json!("Up/Down"));
            m.insert("vaVolume".into(), json!(70));
            m.insert("subscribeRealtime".into(), json!(false));
            m.insert("first_visible_bar_time".into(), Value::Null);
            m.insert("last_visible_bar_time".into(), json!(chrono::Utc::now().timestamp_millis()));
            m
        }
        _ => HashMap::new(),
    }
}
