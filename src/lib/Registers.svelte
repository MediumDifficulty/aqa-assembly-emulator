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

    const { registers }: { registers: Uint32Array } = $props()
</script>

{#each REGISTER_NAMES as name, i}
    <div>
        <span>{name}</span>
        <span>{registers[i]}</span>
    </div>
{/each}