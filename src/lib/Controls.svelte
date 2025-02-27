<script lang="ts">
    import { REGISTERS, FLAGS, RAM } from "./globals";
    import * as engine from "./engine"
    import { get } from "svelte/store";
    import DebugStepOver from "~icons/codicon/debug-step-over"
    import DebugRestart from '~icons/codicon/debug-restart'
    import DebugContinue from '~icons/codicon/debug-continue'

    type ExecutionResult = {
        message: string,
        flags: number
    }

    function stepCpu() {
        const res: ExecutionResult = engine.step(get(RAM), get(REGISTERS), get(FLAGS))
        $FLAGS = res.flags
        REGISTERS.update(v => v)
        RAM.update(v => v)
    }

    function ResetCpu() {
        $FLAGS = 0
        REGISTERS.update(r => r.fill(0))
        RAM.update(r => r.fill(0))
    }
</script>

<div class="tooltip tooltip-bottom" data-tip="Step">
    <button onclick={stepCpu} class="text-success cursor-pointer m-0.5 h-fit"><DebugStepOver /></button>
</div>
<div class="tooltip tooltip-bottom" data-tip="Run (WIP)">
    <button class="text-success cursor-pointer m-0.5 h-fit"><DebugContinue /></button>
</div>
<div class="tooltip tooltip-bottom" data-tip="Reset">
    <button onclick={ResetCpu} class="text-warning cursor-pointer m-0.5 h-fit"><DebugRestart /></button>
</div>

