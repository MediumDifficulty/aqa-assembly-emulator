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
        console.log("step")
        const res: ExecutionResult = engine.step(get(RAM), get(REGISTERS), get(FLAGS))
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

<div class="text-lg self-center mt-1">
    <div class="tooltip tooltip-bottom" data-tip="Step">
        <button onclick={stepCpu} class="text-success cursor-pointer m-0.5"><DebugStepOver /></button>
    </div>
    <div class="tooltip tooltip-bottom" data-tip="Run (WIP)">
        <button class="text-success cursor-pointer m-0.5"><DebugContinue /></button>
    </div>
    <div class="tooltip tooltip-bottom" data-tip="Reset">
        <button onclick={ResetCpu} class="text-warning cursor-pointer m-0.5"><DebugRestart /></button>
    </div>
</div>
