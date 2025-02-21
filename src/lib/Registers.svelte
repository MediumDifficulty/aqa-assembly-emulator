<script lang="ts">
    const REGISTER_NAMES = [
        "R0",
        "R1",
        "R2",
        "R3",
        "R4",
        "R5",
        "R6",
        "R7",
        "R8",
        "R9",
        "R10",
        "R11",
        "R12",
        "SP",
        "LR",
        "PC"
    ]

    const FLAG_NAMES = [
        "N",
        "Z",
        "C",
        "V"
    ]

    const { registers, flags }: { registers: Uint32Array, flags: number } = $props()

</script>

<div class="relative w-full">
    {#each REGISTER_NAMES as name, i}
        <div class="flex justify-between">
            <span>{name}</span>
            <span>{registers[i]}</span>
        </div>
    {/each}
    <div class="flex flex-row justify-between">
        {#each FLAG_NAMES as name, i}
            <span>
                <div>{name}</div>
                <div>{(flags >> i) & 1}</div>
            </span>
        {/each}
    </div>
</div>
