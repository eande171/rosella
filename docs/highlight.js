import hljs from "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/es/core.min.js";
import bash from "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/es/languages/bash.min.js";
import dos from "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/es/languages/dos.min.js";

// Rosella Grammar
function rosella(hljs) {
    return {
        name: "Rosella",
        keywords: {
            keyword: "fn let if else while for in return break continue with",
            type: "int str",
            literal: "windows linux",
            built_in: "print read arg arg_count env set_env number is_number length slice replace upper lower "
                + "concat contains path get_cwd script_dir cd make_dir remove remove_dir copy move write_file "
                + "append_file exists not_exists is_file is_dir files range run run_output random sleep exit",
        },
        contains: [
            hljs.C_BLOCK_COMMENT_MODE,
            { scope: "string", begin: '"', end: '"', contains: [hljs.BACKSLASH_ESCAPE] },
            { scope: "number", match: /\b\d+\b/ },
            { scope: "meta", begin: /\|>/, end: /;(?=[^;]*$)/ },
            { match: [/\bfn\s+(?:(?:int|str)\s+)?/, /[A-Za-z_]\w*/], scope: { 2: "title.function" } },
        ],
    };
}

hljs.registerLanguage("rosella", rosella);
hljs.registerLanguage("bash", bash);
hljs.registerLanguage("batch", dos);

export function highlight(element, text, language) {
    element.innerHTML = hljs.highlight(text, { language }).value;
}

export function highlightAll(selector, language) {
    document.querySelectorAll(selector).forEach((element) => highlight(element, element.textContent, language));
}
