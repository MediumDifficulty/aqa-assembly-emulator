<script lang="ts">
    import loader from "@monaco-editor/loader";
    import { onMount } from "svelte";
    import * as lang from "$lib/aqa_assmbly"
    import init, { step } from "$lib/engine"
    import Memory from "$lib/Memory.svelte";
    import { FLAGS, RAM, RAM_SIZE, REGISTERS } from "$lib/globals";
    import Registers from "$lib/Registers.svelte";
    import { get } from "svelte/store";


    let container: HTMLDivElement
    onMount(async () => {
        await init()

        const monacoEditor = await import("monaco-editor")
        loader.config({ monaco: monacoEditor.default })

        let monaco = await loader.init()
        lang.init(monaco)
        

        const editor = monaco.editor.create(container, {
            theme: "vs-dark",
            automaticLayout: true,
            fontFamily: "JetBrains Mono"
        })
        const model = monaco.editor.createModel(
            "begin:\n\tmov R1, #12\n",
            "aqa-assembly"
        )
        
        lang.initModel(monaco, model)

        editor.setModel(model)
    })

    type ExecutionResult = {
        message: string,
        flags: number
    }

    function stepCpu() {
        console.log("step")
        const res: ExecutionResult = step(get(RAM), get(REGISTERS), get(FLAGS))
        console.log(res)
        $FLAGS = res.flags
        REGISTERS.update(v => v)
        RAM.update(v => v)
        console.log($REGISTERS)
    }

    function ResetCpu() {
        $FLAGS = 0
        REGISTERS.update(r => r.fill(0))
        RAM.update(r => r.fill(0))
    }
</script>

<div class="w-full flex flex-row font-mono">
    <div bind:this={container} class="h-screen w-1/3 relative"></div>
    <Memory memory={$RAM} />
    <div class="w-20">
        <Registers flags={$FLAGS} registers={$REGISTERS} />
    </div>
    <div>
        <button onclick={stepCpu}>Step</button>
        <button onclick={ResetCpu}>Reset</button>
    </div>
</div>
