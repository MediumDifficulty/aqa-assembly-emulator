<script lang="ts">
    import { onMount } from "svelte";
    import { PROGRAM_COUNTER } from "./globals";

    const { memory }: { memory: Uint8Array } = $props()

    const LINE_HEIGHT = 25
    const COLS  = 4 * 4

    const isPrintable = (char: number) => char >= 33 && char <= 126

    let container: HTMLDivElement;

    let containerHeight = $state<number | null>(null)
    const observer = new ResizeObserver(resize => {
        containerHeight = resize[0].contentBoxSize[0].blockSize
    })

    let lines = $derived(() => {
        return !containerHeight ? 20 : containerHeight as number / LINE_HEIGHT
    })

    $effect(() => console.log(containerHeight, lines()))

    onMount(() => {
        observer.observe(document.body)
        containerHeight = container.clientHeight
    })
</script>

<!-- TODO: Fix line alignment -->
<div bind:this={container} class="relative pl-2 h-full">
    <div></div>
    <div>
        {#each Array(Math.floor(lines() - 1.5)).keys() as line}
            <div class="flex flex-row hover:bg-primary hover:text-primary-content font-mono h-[25px]">
                <span class="mr-3 text-neutral-400">
                    {(line * COLS).toString(16).padStart(8, "0")}
                </span>
                <span class="mr-3 relative">
                    {#each Array(COLS).keys().map(i => i + line * COLS) as byteIdx}
                        {#if byteIdx >= memory.length}
                            <span class="text-warning mx-1">00</span>
                        {:else}
                            {@const highlight = byteIdx >= $PROGRAM_COUNTER && byteIdx < $PROGRAM_COUNTER + 4}
                            <span class={[ "text-base-content mx-1", highlight && "text-success" ]}>{new Number(memory[byteIdx]).toString(16).padStart(2, "0")}</span>
                        {/if}
                    {/each}
                </span>
                <span>
                    {#each Array(COLS).keys().map(i => i + line * COLS) as byteIdx}
                        {#if byteIdx >= memory.length}
                            <span class="text-warning">.</span>
                        {:else}
                            <span class="text-base-content">{isPrintable(memory[byteIdx]) ? String.fromCharCode(memory[byteIdx]) : "."}</span>
                        {/if}
                    {/each}
                </span>
            </div>
        {/each}
    </div>
</div>