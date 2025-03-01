import type { Monaco } from "@monaco-editor/loader";
import { editor, MarkerSeverity, type IRange, type languages, type Position } from "monaco-editor";
import * as engine from "./engine/engine";
import { PROGRAM_COUNTER, RAM } from "./globals";
import { get } from "svelte/store";

enum Operand {
    Register,
    Label,
    DataSource
}

const INSTRUCTIONS: {
    name: string;
    args: Operand[];
}[] = [
    {
        name: "LDR",
        args: [
            Operand.Register,
            Operand.DataSource
        ],
    },
    {
        name: "STR",
        args: [
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "ADD",
        args: [
            Operand.Register,
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "SUB",
        args: [
            Operand.Register,
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "MOV",
        args: [
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "CMP",
        args: [
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "B",
        args: [
            Operand.Label
        ]
    },
    {
        name: "BEQ",
        args: [
            Operand.Label
        ]
    },
    {
        name: "BNE",
        args: [
            Operand.Label
        ]
    },
    {
        name: "BGT",
        args: [
            Operand.Label
        ]
    },
    {
        name: "BLT",
        args: [
            Operand.Label
        ]
    },
    {
        name: "AND",
        args: [
            Operand.Register,
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "ORR",
        args: [
            Operand.Register,
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "EOR",
        args: [
            Operand.Register,
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "MVN",
        args: [
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "LSL",
        args: [
            Operand.Register,
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "LSR",
        args: [
            Operand.Register,
            Operand.Register,
            Operand.DataSource
        ]
    },
    {
        name: "HALT",
        args: []
    },
]

function completeOperand(operand: Operand, range: IRange, labels: string[], ctx: Monaco): languages.ProviderResult<languages.CompletionList> {
    switch (operand) {
        case Operand.Register:
            return {
                suggestions: [...Array(13).keys().map(reg => {
                    return {
                        label: `R${reg}`,
                        kind: ctx.languages.CompletionItemKind.Variable,
                        insertText: `R${reg}`,
                        range: range
                    }
                })]
            }
        case Operand.Label:

    
        default:
            break;
    }
}

export function init(ctx: Monaco) {
    ctx.languages.register({
        id: "aqa-assembly"
    })

    ctx.languages.setMonarchTokensProvider("aqa-assembly", LANGUAGE)
    ctx.languages.registerCompletionItemProvider("aqa-assembly", {
        provideCompletionItems: (model, position) => getCompletions(model, position, ctx)
    })
}

export function initModel(ctx: Monaco, model: editor.ITextModel, editor: editor.IStandaloneCodeEditor) {
    let lints: Lints = {
        lints: [],
        source_map: new Map()
    }
    let decorations = editor.createDecorationsCollection([])

    const updateInstructionHighlight = () => {
        let instructionLine = lints.source_map.get(get(PROGRAM_COUNTER))
        if (instructionLine) {
            decorations.set([
                {
                    range: new ctx.Range(instructionLine + 1, 1, instructionLine + 1, 1),
                    options: {
                        isWholeLine: true,
                        className: "bg-success opacity-25"
                    }
                }
            ])
        }
    }

    PROGRAM_COUNTER.subscribe(() => updateInstructionHighlight())

    model.onDidChangeContent(e => {
        const modelValue = model.getValue()
        engine.assemble_into_ram(modelValue, get(RAM))

        RAM.update(r => r)
        lints = engine.lint(modelValue)
        updateInstructionHighlight()

        ctx.editor.setModelMarkers(model, "linter", lints.lints.map(lint => {
            const firstChar = model.getLineFirstNonWhitespaceColumn(lint.line)
            return {
                startLineNumber: lint.line + 1,
                endLineNumber: lint.line + 1,
                startColumn: firstChar + lint.from,
                endColumn: firstChar + lint.to,
                message: lint.err,
                severity: MarkerSeverity.Error
            }
        }))
    })

}

type Lints = {
    source_map: Map<number, number>,
    lints: Lint[]
}

type Lint = {
    err: string,
    from: number,
    to: number,
    line: number
}

function getCompletions(model: editor.ITextModel, position: Position, ctx: Monaco): languages.ProviderResult<languages.CompletionList> {
    const line = model.getLineContent(position.lineNumber)
    const word = model.getWordUntilPosition(position)
    const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn,
    }

    const before = model.getValueInRange({
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: 0,
        endColumn: position.column,
    }).trim().split(" ").filter(s => s.length > 0)

    console.log(before)
    if (before.length === 0) {
        return {
            suggestions: INSTRUCTIONS.map(instruction => {
                return {
                    label: instruction.name,
                    kind: ctx.languages.CompletionItemKind.Function,
                    insertText: instruction.name,
                    range: range
                }
            })
        }
    }

    const instructionName = before[0].toUpperCase()

    const instruction = INSTRUCTIONS.find(instr => instr.name === instructionName)

    if (instruction !== undefined) {
        const labels = model.findMatches("e", true, false, false, null, false)
        console.log(labels)

        return completeOperand(instruction.args[before.length - 1], range, [], ctx)
    }
    return {
        suggestions: [
            // {
            //     label: "mov",
            //     kind: ctx.languages.CompletionItemKind.Variable,
            //     insertText: "move",
            //     range: range
            // }
        ]
    }
}

const LANGUAGE: languages.IMonarchLanguage = {
    ignoreCase: true,
    keywords: [
        "ldr", "str", "add", "sub", "mov", "cmp", "b", "and", "orr", "eor", "mvn", "lsl", "lsr", "halt",
        "beq", "bne", "bgt", "blt"
    ],
    tokenizer: {
        root: [
            {
                regex: /(\/\/|;).*$/,
                action: "comment"
            },
            {
                regex: /^(\w+)(:)$/,
                action: ["tag", "default"],
            },
            [/(R\d+)|PC|LR|SP/, "variable"],
            [/#\d+/, "number"],
            [/\w+/, {
                cases: { "@keywords": "keyword" },
            }],
        ]
    }
}