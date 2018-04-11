use pest::Parser;
use virtual_machine::Register as VMRegister;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("dcpu.pest");
#[derive(Parser)]
#[grammar = "dcpu.pest"]
struct DcpuParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::*;
    #[test]
    fn int_literal() {
        parses_to! {
            parser: DcpuParser,
            input: "-123",
            rule: Rule::int_literal,
            tokens: [
                int_literal(0, 4)
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "123",
            rule: Rule::int_literal,
            tokens: [
                int_literal(0, 3)
            ]
        };
    }

    #[test]
    fn hex_literal() {
        parses_to! {
            parser: DcpuParser,
            input: "#ABCD",
            rule: Rule::hex_literal,
            tokens: [
                hex_literal(0, 5)
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "#FF11",
            rule: Rule::hex_literal,
            tokens: [
                hex_literal(0, 5)
            ]
        };
    }

    #[test]
    fn labels() {
        println!("{:?}", DcpuParser::parse(Rule::label_def, ":foo_bar1234"));
        parses_to! {
            parser: DcpuParser,
            input: ":foo_bar1234",
            rule: Rule::label_def,
            tokens: [
                label_def(0, 12, [ident(1, 12)])
            ]
        };
    }

    #[test]
    fn opcode_single() {
        parses_to! {
            parser: DcpuParser,
            input: "JSR #1234",
            rule: Rule::opcode_single,
            tokens: [
                opcode_single(0, 9, [
                    op_sgl(0, 3, [
                           op_jsr(0, 3)
                    ]),
                    operand(4, 9, [
                        hex_literal(4, 9)
                    ])
                ])
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "JSR 1234",
            rule: Rule::opcode_single,
            tokens: [
                opcode_single(0, 8, [
                    op_sgl(0, 3, [
                           op_jsr(0, 3)
                    ]),
                    operand(4, 8, [
                        int_literal(4, 8)
                    ])
                ])
            ]
        };
    }

    #[test]
    fn opcode_double() {
        parses_to! {
            parser: DcpuParser,
            input: "SET PUSH, [some_label]",
            rule: Rule::opcode_double,
            tokens: [
                opcode_double(0, 22, [
                    op_dbl(0, 3, [op_set(0, 3)]),
                    push_operand(4,8),
                    operand(10, 22, [deref(10, 22, [ident(11, 21)])])
                ])
            ]
        };
        
        parses_to! {
            parser: DcpuParser,
            input: "SET [some_label], #10",
            rule: Rule::opcode_double,
            tokens: [
                opcode_double(0, 21, [
                    op_dbl(0, 3, [op_set(0, 3)]),
                    operand(4, 16, [deref(4, 16, [ident(5, 15)])]),
                    operand(18, 21, [hex_literal(18, 21)])
                ])
            ]
        };
    }
    #[test]
    fn operand() {
        parses_to! {
            parser: DcpuParser,
            input: "some_label",
            rule: Rule::operand,
            tokens: [
                operand(0, 10, [ident(0, 10)])
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "I",
            rule: Rule::operand,
            tokens: [
                operand(0, 1, [register(0, 1)])
            ]
        };
    }

    #[test]
    fn statement() {
        parses_to! {
            parser: DcpuParser,
            input: "SET PC, some_label",
            rule: Rule::statement,
            tokens: [
                statement(0, 18, [
                          opcode_double(0, 18, [
                                op_dbl(0, 3, [op_set(0,3)]),
                                operand(4, 6, [register(4,6)]),
                                operand(8, 18, [ident(8, 18)])
                          ]),
                ])
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: ":some_label",
            rule: Rule::statement,
            tokens: [
                statement(0, 11, [
                          label_def(0, 11, [ident(1, 11)]),
                ])
            ]
        };
        parses_to! {
            parser: DcpuParser,
            input: "SET PC, some_label",
            rule: Rule::statement,
            tokens: [
                statement(0, 18, [
                          opcode_double(0, 18, [
                                op_dbl(0, 3, [op_set(0, 3)]),
                                operand(4, 6, [register(4,6)]),
                                operand(8, 18, [ident(8, 18)])
                          ]),
                ])
            ]
        };
    }
    #[test]
    fn full() {
        parses_to! {
            parser: DcpuParser,
            input: ";testing\n:some_label SET PC, some_label ;another test\ndb \"testing\", 0",
            rule: Rule::input,
            tokens: [
                statements(9, 69, [
                    statement(9, 20, [
                        label_def(9, 20, [ident(10, 20)]),
                    ]),
                    statement(21, 39, [
                        opcode_double(21, 39, [
                            op_dbl(21, 24, [op_set(21, 24)]),
                            operand(25, 27, [register(25, 27)]),
                            operand(29, 39, [ident(29, 39)])
                        ])
                    ]),
                    statement(54, 69, [
                        op_db(54, 69, [
                            db_operand(57, 66, [string(57, 66)]),
                            db_operand(68, 69, [int_literal(68, 69)])
                        ])
                    ])
                ])
            ]
        };
    }
}
