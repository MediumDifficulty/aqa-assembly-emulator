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

<div class="inline-block">
    {#each REGISTER_NAMES as name, i}
        <div>{name}</div>
    {/each}    
</div>
<div class="inline-block">
    {#each REGISTER_NAMES as _, i}
        <div>{registers[i]}</div>
    {/each}    
</div>
<div class="">
    {#each FLAG_NAMES as name, i}
        <div class="inline-block mr-1">
            <div>{name}</div>
            <div>{(flags >> (3 - i)) & 1}</div>
        </div>
    {/each}
</div>
