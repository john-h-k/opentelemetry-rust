#[cfg(any(feature = "trace", feature = "logs", feature = "metrics"))]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(any(feature = "trace", feature = "logs", feature = "metrics"))]
pub(crate) fn to_nanos(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos() as u64
}

#[cfg(feature = "gen-tonic-messages")]
pub mod tonic {
    use crate::proto::tonic::common::v1::{
        any_value, AnyValue, ArrayValue, InstrumentationScope, KeyValue,
    };
    use opentelemetry_api::{Array, Value};
    use std::borrow::Cow;

    #[cfg(any(feature = "trace", feature = "logs"))]
    use opentelemetry_sdk::Resource;

    impl From<opentelemetry_sdk::InstrumentationLibrary> for InstrumentationScope {
        fn from(library: opentelemetry_sdk::InstrumentationLibrary) -> Self {
            InstrumentationScope {
                name: library.name.into_owned(),
                version: library.version.map(Cow::into_owned).unwrap_or_default(),
                attributes: Attributes::from(library.attributes).0,
                ..Default::default()
            }
        }
    }

    impl From<&opentelemetry_sdk::InstrumentationLibrary> for InstrumentationScope {
        fn from(library: &opentelemetry_sdk::InstrumentationLibrary) -> Self {
            InstrumentationScope {
                name: library.name.to_string(),
                version: library
                    .version
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                attributes: Attributes::from(library.attributes.clone()).0,
                ..Default::default()
            }
        }
    }

    /// Wrapper type for Vec<[`KeyValue`](crate::proto::tonic::common::v1::KeyValue)>
    #[derive(Default)]
    pub struct Attributes(pub ::std::vec::Vec<crate::proto::tonic::common::v1::KeyValue>);

    #[cfg(feature = "trace")]
    impl From<opentelemetry_sdk::trace::EvictedHashMap> for Attributes {
        fn from(attributes: opentelemetry_sdk::trace::EvictedHashMap) -> Self {
            Attributes(
                attributes
                    .into_iter()
                    .map(|(key, value)| KeyValue {
                        key: key.as_str().to_string(),
                        value: Some(value.into()),
                    })
                    .collect(),
            )
        }
    }

    impl From<Vec<opentelemetry_api::KeyValue>> for Attributes {
        fn from(kvs: Vec<opentelemetry_api::KeyValue>) -> Self {
            Attributes(
                kvs.into_iter()
                    .map(|api_kv| KeyValue {
                        key: api_kv.key.as_str().to_string(),
                        value: Some(api_kv.value.into()),
                    })
                    .collect(),
            )
        }
    }

    #[cfg(feature = "logs")]
    impl<K: Into<String>, V: Into<AnyValue>> FromIterator<(K, V)> for Attributes {
        fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
            Attributes(
                iter.into_iter()
                    .map(|(k, v)| KeyValue {
                        key: k.into(),
                        value: Some(v.into()),
                    })
                    .collect(),
            )
        }
    }

    impl From<Value> for AnyValue {
        fn from(value: Value) -> Self {
            AnyValue {
                value: match value {
                    Value::Bool(val) => Some(any_value::Value::BoolValue(val)),
                    Value::I64(val) => Some(any_value::Value::IntValue(val)),
                    Value::F64(val) => Some(any_value::Value::DoubleValue(val)),
                    Value::String(val) => Some(any_value::Value::StringValue(val.to_string())),
                    Value::Array(array) => Some(any_value::Value::ArrayValue(match array {
                        Array::Bool(vals) => array_into_proto(vals),
                        Array::I64(vals) => array_into_proto(vals),
                        Array::F64(vals) => array_into_proto(vals),
                        Array::String(vals) => array_into_proto(vals),
                    })),
                },
            }
        }
    }

    fn array_into_proto<T>(vals: Vec<T>) -> ArrayValue
    where
        Value: From<T>,
    {
        let values = vals
            .into_iter()
            .map(|val| AnyValue::from(Value::from(val)))
            .collect();

        ArrayValue { values }
    }

    #[cfg(any(feature = "trace", feature = "logs"))]
    pub(crate) fn resource_attributes(resource: &Resource) -> Attributes {
        resource
            .iter()
            .map(|(k, v)| opentelemetry_api::KeyValue::new(k.clone(), v.clone()))
            .collect::<Vec<_>>()
            .into()
    }
}

#[cfg(feature = "gen-protoc")]
pub mod grpcio {
    use crate::proto::grpcio::common::{AnyValue, ArrayValue, InstrumentationScope, KeyValue};
    use opentelemetry_api::{Array, Value};
    use opentelemetry_sdk::Resource;
    use protobuf::RepeatedField;
    #[cfg(feature = "logs")]
    use protobuf::SingularPtrField;
    use std::borrow::Cow;

    impl From<opentelemetry_sdk::InstrumentationLibrary> for InstrumentationScope {
        fn from(library: opentelemetry_sdk::InstrumentationLibrary) -> Self {
            InstrumentationScope {
                name: library.name.into_owned(),
                version: library.version.map(Cow::into_owned).unwrap_or_default(),
                attributes: Attributes::from(library.attributes).0,
                ..Default::default()
            }
        }
    }

    #[derive(Default)]
    pub struct Attributes(pub ::protobuf::RepeatedField<crate::proto::grpcio::common::KeyValue>);

    #[cfg(feature = "trace")]
    impl From<opentelemetry_sdk::trace::EvictedHashMap> for Attributes {
        fn from(attributes: opentelemetry_sdk::trace::EvictedHashMap) -> Self {
            Attributes(RepeatedField::from_vec(
                attributes
                    .into_iter()
                    .map(|(key, value)| {
                        let mut kv: KeyValue = KeyValue::new();
                        kv.set_key(key.as_str().to_string());
                        kv.set_value(value.into());
                        kv
                    })
                    .collect(),
            ))
        }
    }

    impl From<Vec<opentelemetry_api::KeyValue>> for Attributes {
        fn from(kvs: Vec<opentelemetry_api::KeyValue>) -> Self {
            Attributes(RepeatedField::from_vec(
                kvs.into_iter()
                    .map(|api_kv| {
                        let mut kv: KeyValue = KeyValue::new();
                        kv.set_key(api_kv.key.as_str().to_string());
                        kv.set_value(api_kv.value.into());
                        kv
                    })
                    .collect(),
            ))
        }
    }

    #[cfg(feature = "logs")]
    impl<K: Into<String>, V: Into<AnyValue>> FromIterator<(K, V)> for Attributes {
        fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
            Attributes(RepeatedField::from_vec(
                iter.into_iter()
                    .map(|(k, v)| KeyValue {
                        key: k.into(),
                        value: SingularPtrField::some(v.into()),
                        ..Default::default()
                    })
                    .collect(),
            ))
        }
    }

    impl From<Value> for AnyValue {
        fn from(value: Value) -> Self {
            let mut any_value = AnyValue::new();
            match value {
                Value::Bool(val) => any_value.set_bool_value(val),
                Value::I64(val) => any_value.set_int_value(val),
                Value::F64(val) => any_value.set_double_value(val),
                Value::String(val) => any_value.set_string_value(val.to_string()),
                Value::Array(array) => any_value.set_array_value(match array {
                    Array::Bool(vals) => array_into_proto(vals),
                    Array::I64(vals) => array_into_proto(vals),
                    Array::F64(vals) => array_into_proto(vals),
                    Array::String(vals) => array_into_proto(vals),
                }),
            };

            any_value
        }
    }

    fn array_into_proto<T>(vals: Vec<T>) -> ArrayValue
    where
        Value: From<T>,
    {
        let values = RepeatedField::from_vec(
            vals.into_iter()
                .map(|val| AnyValue::from(Value::from(val)))
                .collect(),
        );

        let mut array_value = ArrayValue::new();
        array_value.set_values(values);
        array_value
    }

    #[cfg(any(feature = "trace", feature = "logs"))]
    pub(crate) fn resource_attributes(resource: &Resource) -> Attributes {
        resource
            .iter()
            .map(|(k, v)| opentelemetry_api::KeyValue::new(k.clone(), v.clone()))
            .collect::<Vec<_>>()
            .into()
    }
}
