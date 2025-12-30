#[cfg(test)]
mod tests {
    use crate::sqlz::GenericType;
    use crate::sqlz::schema::{Column, Table};
    use std::rc::Rc;

    fn create_column(name: &str, col_type: GenericType) -> Rc<Column> {
        Rc::new(Column {
            name: name.to_string(),
            col_type,
            native_type: "".to_string(),
            is_nullable: false,
            is_identity: false,
            max_length: None,
            numeric_precision: None,
            numeric_scale: None,
        })
    }

    fn create_table(name: &str, columns: Vec<Rc<Column>>) -> Table {
        Table {
            name: name.to_string(),
            schema: None,
            columns,
            constraints: vec![],
        }
    }

    #[test]
    fn test_fewer_columns_fails() {
        let col1 = create_column("col1", GenericType::Integer);
        let table1 = create_table("table1", vec![col1.clone(), col1.clone()]);
        let table2 = create_table("table2", vec![col1]);
        assert!(!table1.can_migrate_to(&table2));
    }

    #[test]
    fn test_numeric_widening() {
        let types = vec![
            GenericType::TinyInt,
            GenericType::SmallInt,
            GenericType::Integer,
            GenericType::BigInt,
        ];

        for (i, source_type) in types.iter().enumerate() {
            for target_type in types.iter().skip(i) {
                let source = create_table("s", vec![create_column("c", source_type.clone())]);
                let target = create_table("t", vec![create_column("c", target_type.clone())]);
                assert!(
                    source.can_migrate_to(&target),
                    "Failed: {:?} -> {:?}",
                    source_type,
                    target_type
                );
            }
        }
    }

    #[test]
    fn test_numeric_narrowing_fails() {
        let source = create_table("s", vec![create_column("c", GenericType::BigInt)]);
        let target = create_table("t", vec![create_column("c", GenericType::Integer)]);
        assert!(!source.can_migrate_to(&target));
    }

    #[test]
    fn test_float_widening() {
        let f = create_table("s", vec![create_column("c", GenericType::Float)]);
        let d = create_table("t", vec![create_column("c", GenericType::Double)]);
        assert!(f.can_migrate_to(&d));
        assert!(f.can_migrate_to(&f));
        assert!(d.can_migrate_to(&d));
        assert!(!d.can_migrate_to(&f));
    }

    #[test]
    fn test_string_rules() {
        // Text/UserDefined -> Text
        let text = create_table("s", vec![create_column("c", GenericType::Text)]);
        let ud = create_table(
            "s",
            vec![create_column(
                "c",
                GenericType::UserDefined("custom".into()),
            )],
        );
        assert!(text.can_migrate_to(&text));
        assert!(ud.can_migrate_to(&text));
        assert!(!text.can_migrate_to(&ud));

        // VarChar size checks
        let v10 = create_table("s", vec![create_column("c", GenericType::VarChar(10))]);
        let v20 = create_table("t", vec![create_column("c", GenericType::VarChar(20))]);
        let v5 = create_table("t", vec![create_column("c", GenericType::VarChar(5))]);
        assert!(v10.can_migrate_to(&v20));
        assert!(v10.can_migrate_to(&v10));
        assert!(!v10.can_migrate_to(&v5));

        // Char size checks
        let c10 = create_table("s", vec![create_column("c", GenericType::Char(10))]);
        let c20 = create_table("t", vec![create_column("c", GenericType::Char(20))]);
        assert!(c10.can_migrate_to(&c20));
        assert!(!c10.can_migrate_to(&create_table(
            "t",
            vec![create_column("c", GenericType::Char(5))]
        )));

        // VarChar/Char -> Text
        assert!(v10.can_migrate_to(&text));
        assert!(c10.can_migrate_to(&text));
    }

    #[test]
    fn test_binary_bit_rules() {
        let b10 = create_table("s", vec![create_column("c", GenericType::Blob(10))]);
        let b20 = create_table("t", vec![create_column("c", GenericType::Blob(20))]);
        assert!(b10.can_migrate_to(&b20));
        assert!(!b20.can_migrate_to(&b10));

        let bit10 = create_table("s", vec![create_column("c", GenericType::Bit(10))]);
        let bit20 = create_table("t", vec![create_column("c", GenericType::Bit(20))]);
        assert!(bit10.can_migrate_to(&bit20));

        let vbit10 = create_table("s", vec![create_column("c", GenericType::VarBit(10))]);
        let vbit20 = create_table("t", vec![create_column("c", GenericType::VarBit(20))]);
        assert!(vbit10.can_migrate_to(&vbit20));
    }

    #[test]
    fn test_decimal_rules() {
        let d10_2 = create_table(
            "s",
            vec![create_column(
                "c",
                GenericType::Decimal {
                    precision: 10,
                    scale: 2,
                },
            )],
        );
        let d20_2 = create_table(
            "t",
            vec![create_column(
                "c",
                GenericType::Decimal {
                    precision: 20,
                    scale: 2,
                },
            )],
        );
        let d20_3 = create_table(
            "t",
            vec![create_column(
                "c",
                GenericType::Decimal {
                    precision: 20,
                    scale: 3,
                },
            )],
        );

        assert!(d10_2.can_migrate_to(&d20_2));
        assert!(!d10_2.can_migrate_to(&d20_3)); // Scale must match exactly based on current logic
        assert!(!d20_2.can_migrate_to(&d10_2)); // Precision must be >=
    }

    #[test]
    fn test_exact_match_others() {
        let cases = vec![
            GenericType::Boolean,
            GenericType::Date,
            GenericType::Timestamp,
            GenericType::TimestampTz,
            GenericType::Time,
            GenericType::Json,
            GenericType::Uuid,
        ];

        for t in cases {
            let s = create_table("s", vec![create_column("c", t.clone())]);
            let target = create_table("t", vec![create_column("c", t.clone())]);
            assert!(s.can_migrate_to(&target));

            let other = create_table("o", vec![create_column("c", GenericType::Integer)]);
            assert!(!s.can_migrate_to(&other));
        }
    }
}
