// Rosella Compiler

const encoder = new TextEncoder();
const decoder = new TextDecoder();
let loading = null;

export function loadCompiler(url) {
    loading ??= WebAssembly.instantiateStreaming(fetch(url), {}).then(({ instance }) => {
        const wasm = instance.exports;
        return {
            compile(source, shell) {
                const input = encoder.encode(source);
                const pointer = wasm.alloc(input.length);
                new Uint8Array(wasm.memory.buffer, pointer, input.length).set(input);
                const ok = wasm.compile(pointer, input.length, shell === "batch" ? 0 : 1) === 1;
                wasm.dealloc(pointer, input.length);
                const output = new Uint8Array(wasm.memory.buffer, wasm.output_pointer(), wasm.output_length());
                return { ok, text: decoder.decode(output).replace(/\r\n/g, "\n") };
            },
        };
    });
    return loading;
}
