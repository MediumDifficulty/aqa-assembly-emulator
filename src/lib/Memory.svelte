<script lang="ts">
    const { memory }: { memory: Uint8Array } = $props()

    const LINES = 50
    const COLS  = 4 * 4

    const isPrintable = (char: number) => char >= 33 && char <= 126
</script>

<div class="max-h-screen overflow-auto relative pl-2">
    <!-- <div class="sticky top-1 bg-white border-gray-500 border-1">
        <span>Page: </span>
        <input class="" type="number" name="" id="" maxlength="4">
    </div> -->
    {#each Array(LINES).keys() as line}
        <div class="flex flex-row hover:bg-gray-200 font-mono">
            <span class="mr-3 text-gray-600">
                {(line * COLS).toString(16).padStart(8, "0")}
            </span>
            <span class="mr-3">
                {#each Array(COLS).keys().map(i => i + line * COLS) as byteIdx}
                    {#if byteIdx >= memory.length}
                        <span class="text-red-600 mx-1">00</span>
                    {:else}
                        <span class="mx-1">{new Number(memory[byteIdx]).toString(16).padStart(2, "0")}</span>
                    {/if}
                {/each}
            </span>
            <span>
                {#each Array(COLS).keys().map(i => i + line * COLS) as byteIdx}
                    {#if byteIdx >= memory.length}
                        <span class="text-red-600">.</span>
                    {:else}
                        <span class="">{isPrintable(memory[byteIdx]) ? String.fromCharCode(memory[byteIdx]) : "."}</span>
                    {/if}
                {/each}
            </span>
        </div>
    {/each}
</div>