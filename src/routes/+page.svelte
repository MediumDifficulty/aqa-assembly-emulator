<script lang="ts">
    import loader from "@monaco-editor/loader";
    import { onMount } from "svelte";
    import * as lang from "$lib/aqa_assmbly"
    import init, { step } from "$lib/engine"
    import Memory from "$lib/Memory.svelte";
    import { FLAGS, RAM, REGISTERS } from "$lib/globals";
    import Registers from "$lib/Registers.svelte";
    import * as monacoEditor from 'monaco-editor';
    import Controls from "$lib/Controls.svelte";

    let container: HTMLDivElement
    let header: HTMLHeadElement
    let editor: monacoEditor.editor.IStandaloneCodeEditor

    onMount(async () => {
        await init()

        loader.config({ monaco: monacoEditor })

        let monaco = await loader.init()
        lang.init(monaco)
        

        editor = monaco.editor.create(container, {
            theme: "vs-dark",
            fontFamily: "JetBrains Mono"
        })
        const model = monaco.editor.createModel(
            "begin:\n\tmov R1, #12\n",
            "aqa-assembly"
        )
        
        lang.initModel(monaco, model)

        editor.setModel(model)
    })

    const updateEditorSize = () => {
        // TODO: Try to do this with CSS
        const height = window.innerHeight - header.offsetHeight
        editor.layout({
            width: container.offsetWidth,
            height: height
        })
    }

</script>

<svelte:window 
    on:resize={updateEditorSize}
/>

<div class="h-screen max-h-screen flex flex-col">
    <header bind:this={header} class="bg-base-300 relative w-full">
        <div class="mx-auto w-fit text-lg mt-1">
            <Controls />
        </div>
    </header>
    <div class="w-full flex flex-row font-mono justify-between h-full relative">
        <div bind:this={container} class="w-1/3 h-full"></div>
        <div class="grow">
            <Registers flags={$FLAGS} registers={$REGISTERS} />
        </div>
        <Memory memory={$RAM} />
    </div>
</div>
