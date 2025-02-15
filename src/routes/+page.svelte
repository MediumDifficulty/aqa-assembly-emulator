<script lang="ts">
    import loader from "@monaco-editor/loader";
    import * as Monaco from "monaco-editor"
    import { onMount } from "svelte";
    import * as lang from "$lib/aqa_assmbly"
    import init, { test } from "$lib/engine"
    // let { monacoEditor }: { monacoEditor: typeof Monaco } = $props()


    let container: HTMLDivElement
    onMount(async () => {
        await init()
        console.log(test())

        const monacoEditor = await import("monaco-editor")
        loader.config({ monaco: monacoEditor.default })

        let monaco = await loader.init()
        lang.init(monaco)
        

        const editor = monaco.editor.create(container, {
            theme: "vs-dark",
            automaticLayout: true
        })
        const model = monaco.editor.createModel(
            "begin:\n\tmov R1, #12\n",
            "aqa-assembly"
        )
        
        lang.initModel(monaco, model)

        editor.setModel(model)
    })
    
</script>

<!-- {#await import('monaco-editor')}
    <div>Loading...</div>
{:then monaco}  -->
    <!-- <Editor /> -->
<!-- {/await} -->
<div bind:this={container} class="h-screen w-[50%] relative"></div>
