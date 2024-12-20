let wasm_bindgen;
(function() {
    const __exports = {};
    let script_src;
    if (typeof document !== 'undefined' && document.currentScript !== null) {
        script_src = new URL(document.currentScript.src, location.href).toString();
    }
    let wasm = undefined;

    const heap = new Array(128).fill(undefined);

    heap.push(undefined, null, true, false);

    function getObject(idx) { return heap[idx]; }

    const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

    if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

    let cachedUint8ArrayMemory0 = null;

    function getUint8ArrayMemory0() {
        if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
            cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
        }
        return cachedUint8ArrayMemory0;
    }

    function getStringFromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
    }

    let heap_next = heap.length;

    function addHeapObject(obj) {
        if (heap_next === heap.length) heap.push(heap.length + 1);
        const idx = heap_next;
        heap_next = heap[idx];

        heap[idx] = obj;
        return idx;
    }

    function handleError(f, args) {
        try {
            return f.apply(this, args);
        } catch (e) {
            wasm.__wbindgen_exn_store(addHeapObject(e));
        }
    }

    function dropObject(idx) {
        if (idx < 132) return;
        heap[idx] = heap_next;
        heap_next = idx;
    }

    function takeObject(idx) {
        const ret = getObject(idx);
        dropObject(idx);
        return ret;
    }

    let WASM_VECTOR_LEN = 0;

    const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

    const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
        ? function (arg, view) {
        return cachedTextEncoder.encodeInto(arg, view);
    }
        : function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    });

    function passStringToWasm0(arg, malloc, realloc) {

        if (realloc === undefined) {
            const buf = cachedTextEncoder.encode(arg);
            const ptr = malloc(buf.length, 1) >>> 0;
            getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
            WASM_VECTOR_LEN = buf.length;
            return ptr;
        }

        let len = arg.length;
        let ptr = malloc(len, 1) >>> 0;

        const mem = getUint8ArrayMemory0();

        let offset = 0;

        for (; offset < len; offset++) {
            const code = arg.charCodeAt(offset);
            if (code > 0x7F) break;
            mem[ptr + offset] = code;
        }

        if (offset !== len) {
            if (offset !== 0) {
                arg = arg.slice(offset);
            }
            ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
            const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
            const ret = encodeString(arg, view);

            offset += ret.written;
            ptr = realloc(ptr, len, offset, 1) >>> 0;
        }

        WASM_VECTOR_LEN = offset;
        return ptr;
    }

    let cachedDataViewMemory0 = null;

    function getDataViewMemory0() {
        if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
            cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
        }
        return cachedDataViewMemory0;
    }

    function isLikeNone(x) {
        return x === undefined || x === null;
    }

    function getArrayJsValueFromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        const mem = getDataViewMemory0();
        const result = [];
        for (let i = ptr; i < ptr + 4 * len; i += 4) {
            result.push(takeObject(mem.getUint32(i, true)));
        }
        return result;
    }

    function getArrayU8FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
    }

    const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(state => {
        wasm.__wbindgen_export_4.get(state.dtor)(state.a, state.b)
    });

    function makeMutClosure(arg0, arg1, dtor, f) {
        const state = { a: arg0, b: arg1, cnt: 1, dtor };
        const real = (...args) => {
            // First up with a closure we increment the internal reference
            // count. This ensures that the Rust closure environment won't
            // be deallocated while we're invoking it.
            state.cnt++;
            const a = state.a;
            state.a = 0;
            try {
                return f(a, state.b, ...args);
            } finally {
                if (--state.cnt === 0) {
                    wasm.__wbindgen_export_4.get(state.dtor)(a, state.b);
                    CLOSURE_DTORS.unregister(state);
                } else {
                    state.a = a;
                }
            }
        };
        real.original = state;
        CLOSURE_DTORS.register(real, state, state);
        return real;
    }

    function makeClosure(arg0, arg1, dtor, f) {
        const state = { a: arg0, b: arg1, cnt: 1, dtor };
        const real = (...args) => {
            // First up with a closure we increment the internal reference
            // count. This ensures that the Rust closure environment won't
            // be deallocated while we're invoking it.
            state.cnt++;
            try {
                return f(state.a, state.b, ...args);
            } finally {
                if (--state.cnt === 0) {
                    wasm.__wbindgen_export_4.get(state.dtor)(state.a, state.b);
                    state.a = 0;
                    CLOSURE_DTORS.unregister(state);
                }
            }
        };
        real.original = state;
        CLOSURE_DTORS.register(real, state, state);
        return real;
    }

    function debugString(val) {
        // primitive types
        const type = typeof val;
        if (type == 'number' || type == 'boolean' || val == null) {
            return  `${val}`;
        }
        if (type == 'string') {
            return `"${val}"`;
        }
        if (type == 'symbol') {
            const description = val.description;
            if (description == null) {
                return 'Symbol';
            } else {
                return `Symbol(${description})`;
            }
        }
        if (type == 'function') {
            const name = val.name;
            if (typeof name == 'string' && name.length > 0) {
                return `Function(${name})`;
            } else {
                return 'Function';
            }
        }
        // objects
        if (Array.isArray(val)) {
            const length = val.length;
            let debug = '[';
            if (length > 0) {
                debug += debugString(val[0]);
            }
            for(let i = 1; i < length; i++) {
                debug += ', ' + debugString(val[i]);
            }
            debug += ']';
            return debug;
        }
        // Test for built-in
        const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
        let className;
        if (builtInMatches && builtInMatches.length > 1) {
            className = builtInMatches[1];
        } else {
            // Failed to match the standard '[object ClassName]'
            return toString.call(val);
        }
        if (className == 'Object') {
            // we're a user defined class or Object
            // JSON.stringify avoids problems with cycles, and is generally much
            // easier than looping through ownProperties of `val`.
            try {
                return 'Object(' + JSON.stringify(val) + ')';
            } catch (_) {
                return 'Object';
            }
        }
        // errors
        if (val instanceof Error) {
            return `${val.name}: ${val.message}\n${val.stack}`;
        }
        // TODO we could test for more things here, like `Set`s and `Map`s.
        return className;
    }
    function __wbg_adapter_50(arg0, arg1) {
        wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h6ee4aa4a1d1d55e6(arg0, arg1);
    }

    function __wbg_adapter_53(arg0, arg1, arg2) {
        wasm._dyn_core__ops__function__Fn__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hc9455182c31b3061(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_56(arg0, arg1, arg2) {
        wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0a958cab54d3cff4(arg0, arg1, addHeapObject(arg2));
    }

    let stack_pointer = 128;

    function addBorrowedObject(obj) {
        if (stack_pointer == 1) throw new Error('out of js stack');
        heap[--stack_pointer] = obj;
        return stack_pointer;
    }
    function __wbg_adapter_61(arg0, arg1, arg2) {
        try {
            wasm._dyn_core__ops__function__FnMut___A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hb94ea77bac0171c4(arg0, arg1, addBorrowedObject(arg2));
        } finally {
            heap[stack_pointer++] = undefined;
        }
    }

    function __wbg_adapter_64(arg0, arg1, arg2) {
        wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0734078ec4fc9b08(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_67(arg0, arg1, arg2) {
        try {
            wasm.wasm_bindgen__convert__closures__invoke1_mut_ref__h2099b6b87bdcee76(arg0, arg1, addBorrowedObject(arg2));
        } finally {
            heap[stack_pointer++] = undefined;
        }
    }

    function __wbg_adapter_70(arg0, arg1, arg2) {
        wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h14c20855869e2d1e(arg0, arg1, addHeapObject(arg2));
    }

    const __wbindgen_enum_BinaryType = ["blob", "arraybuffer"];

    const __wbindgen_enum_CodecState = ["unconfigured", "configured", "closed"];

    const __wbindgen_enum_EncodedAudioChunkType = ["key", "delta"];

    const __wbindgen_enum_EncodedVideoChunkType = ["key", "delta"];

    const __wbindgen_enum_LatencyMode = ["quality", "realtime"];

    async function __wbg_load(module, imports) {
        if (typeof Response === 'function' && module instanceof Response) {
            if (typeof WebAssembly.instantiateStreaming === 'function') {
                try {
                    return await WebAssembly.instantiateStreaming(module, imports);

                } catch (e) {
                    if (module.headers.get('Content-Type') != 'application/wasm') {
                        console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                    } else {
                        throw e;
                    }
                }
            }

            const bytes = await module.arrayBuffer();
            return await WebAssembly.instantiate(bytes, imports);

        } else {
            const instance = await WebAssembly.instantiate(module, imports);

            if (instance instanceof WebAssembly.Instance) {
                return { instance, module };

            } else {
                return instance;
            }
        }
    }

    function __wbg_get_imports() {
        const imports = {};
        imports.wbg = {};
        imports.wbg.__wbg_addEventListener_37872d53aeb4c65a = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), getObject(arg4));
        }, arguments) };
        imports.wbg.__wbg_buffer_61b7ce01341d7f88 = function(arg0) {
            const ret = getObject(arg0).buffer;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_byteLength_98484655a4827f35 = function(arg0) {
            const ret = getObject(arg0).byteLength;
            return ret;
        };
        imports.wbg.__wbg_byteLength_c19ec7d7cfaa5fe0 = function(arg0) {
            const ret = getObject(arg0).byteLength;
            return ret;
        };
        imports.wbg.__wbg_call_500db948e69c7330 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_call_b0d8e36992d9900d = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_catch_d0fc80129c999ab3 = function(arg0, arg1) {
            const ret = getObject(arg0).catch(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_clearInterval_d0ff292406f98cc3 = function(arg0) {
            const ret = clearInterval(takeObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_close_1b080924f944a36d = function(arg0) {
            getObject(arg0).close();
        };
        imports.wbg.__wbg_close_1d787025dbecab6e = function() { return handleError(function (arg0) {
            getObject(arg0).close();
        }, arguments) };
        imports.wbg.__wbg_close_2a6496dda8cb95f9 = function(arg0) {
            getObject(arg0).close();
        };
        imports.wbg.__wbg_close_2ec3ef800dd8382b = function() { return handleError(function (arg0) {
            getObject(arg0).close();
        }, arguments) };
        imports.wbg.__wbg_close_4063e1bcbd6d5fe2 = function() { return handleError(function (arg0) {
            getObject(arg0).close();
        }, arguments) };
        imports.wbg.__wbg_close_6e2eb664156b0e0c = function(arg0) {
            const ret = getObject(arg0).close();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_close_9bb0651568a4466f = function(arg0) {
            getObject(arg0).close();
        };
        imports.wbg.__wbg_close_b5a81c021f7bcfc2 = function(arg0, arg1) {
            getObject(arg0).close(getObject(arg1));
        };
        imports.wbg.__wbg_closed_8626b258213020cb = function(arg0) {
            const ret = getObject(arg0).closed;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_configure_0922d796fc673316 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).configure(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_configure_69db7caaa1be44b3 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).configure(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_configure_917f0ab4e47ea724 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).configure(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_configure_dbc4f153992d75e2 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).configure(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_copyTo_2425bb66f47c74e9 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).copyTo(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_copyTo_d78ffe79fd8bc330 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).copyTo(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_createObjectURL_296ad2113ed20fe0 = function() { return handleError(function (arg0, arg1) {
            const ret = URL.createObjectURL(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_createUnidirectionalStream_92e83128972fa9d2 = function(arg0) {
            const ret = getObject(arg0).createUnidirectionalStream();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_crypto_805be4ce92f1e370 = function(arg0) {
            const ret = getObject(arg0).crypto;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_data_4ce8a82394d8b110 = function(arg0) {
            const ret = getObject(arg0).data;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_datagrams_32f09c51a613fa05 = function(arg0) {
            const ret = getObject(arg0).datagrams;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_decode_3be6571fb9b3b6b5 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).decode(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_decode_c7e1289356e5984a = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).decode(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_done_f22c1561fa919baa = function(arg0) {
            const ret = getObject(arg0).done;
            return ret;
        };
        imports.wbg.__wbg_duration_8e683ee218462673 = function(arg0, arg1) {
            const ret = getObject(arg1).duration;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        };
        imports.wbg.__wbg_duration_fab7baf2eee49d42 = function(arg0, arg1) {
            const ret = getObject(arg1).duration;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        };
        imports.wbg.__wbg_encode_01f2fb7cbb24d1a1 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).encode(getObject(arg1), getObject(arg2));
        }, arguments) };
        imports.wbg.__wbg_encode_388d77d48a4bc30d = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).encode(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_entries_4f2bb9b0d701c0f6 = function(arg0) {
            const ret = Object.entries(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_error_fab41a42d22bf2bc = function(arg0) {
            console.error(getObject(arg0));
        };
        imports.wbg.__wbg_from_d68eaa96dba25449 = function(arg0) {
            const ret = Array.from(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getRandomValues_f6a868620c8bab49 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).getRandomValues(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_getReader_20b92737617d61a1 = function(arg0) {
            const ret = getObject(arg0).getReader();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_getWriter_dd1c7a1972bcd348 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).getWriter();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_get_9aa3dff3f0266054 = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_get_bbccf8970793c087 = function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_getwithrefkey_6550b2c093d2eb18 = function(arg0, arg1) {
            const ret = getObject(arg0)[getObject(arg1)];
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_incomingBidirectionalStreams_e0f705b3054fac53 = function(arg0) {
            const ret = getObject(arg0).incomingBidirectionalStreams;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_incomingUnidirectionalStreams_388b8eefdae2881f = function(arg0) {
            const ret = getObject(arg0).incomingUnidirectionalStreams;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_instanceof_ArrayBuffer_670ddde44cdb2602 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Array_7707ae9a3c3dd458 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_AudioData_79bc633c3edbd0df = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof AudioData;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_MessageEvent_4503e23b62c66dfc = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof MessageEvent;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Uint8Array_28af5bc19d6acad8 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_VideoFrame_e8a11c8d82b5c1ef = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof VideoFrame;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Window_d2514c6a7ee7ba60 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_isArray_1ba11a930108ec51 = function(arg0) {
            const ret = Array.isArray(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_isSafeInteger_12f5549b2fca23f4 = function(arg0) {
            const ret = Number.isSafeInteger(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_iterator_23604bb983791576 = function() {
            const ret = Symbol.iterator;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_length_65d1cd11729ced11 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_length_d65cf0786bfc5739 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_location_b2ec7e36fec8a8ff = function(arg0) {
            const ret = getObject(arg0).location;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_locked_305ddbf5519a81be = function(arg0) {
            const ret = getObject(arg0).locked;
            return ret;
        };
        imports.wbg.__wbg_log_464d1b2190ca1e04 = function(arg0) {
            console.log(getObject(arg0));
        };
        imports.wbg.__wbg_log_e51ef223c244b133 = function(arg0, arg1) {
            var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice();
            wasm.__wbindgen_free(arg0, arg1 * 4, 4);
            console.log(...v0);
        };
        imports.wbg.__wbg_msCrypto_2ac4d17c4748234a = function(arg0) {
            const ret = getObject(arg0).msCrypto;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_053f965e0edbfedf = function() { return handleError(function (arg0) {
            const ret = new AudioEncoder(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_254fa9eac11932ae = function() {
            const ret = new Array();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_3d662e5b2b6548c5 = function() { return handleError(function (arg0, arg1) {
            const ret = new Worker(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_3ff5b33b1ce712df = function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_4d472939256be363 = function() { return handleError(function (arg0) {
            const ret = new VideoDecoder(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_5b567da05d91219b = function() { return handleError(function (arg0) {
            const ret = new AudioDecoder(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_688846f374351c92 = function() {
            const ret = new Object();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_6aa25527a5efd124 = function() { return handleError(function (arg0) {
            const ret = new EncodedVideoChunk(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_78f2ceff39aae5cd = function() { return handleError(function (arg0) {
            const ret = new EncodedAudioChunk(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_93e11344654eb7bd = function() { return handleError(function (arg0, arg1) {
            const ret = new WebTransport(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_9b6c38191d7b9512 = function() { return handleError(function (arg0, arg1) {
            const ret = new WebSocket(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_f52eb581bb023e07 = function() { return handleError(function (arg0) {
            const ret = new VideoEncoder(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_newnoargs_fd9e4bf8be2bc16d = function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_ba35896968751d91 = function(arg0, arg1, arg2) {
            const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithlength_34ce8f1051e74449 = function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithstrsequenceandoptions_c12c1efe3dd90e2c = function() { return handleError(function (arg0, arg1) {
            const ret = new Blob(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_next_01dd9234a5bf6d05 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).next();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_next_137428deb98342b0 = function(arg0) {
            const ret = getObject(arg0).next;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_node_ecc8306b9857f33d = function(arg0) {
            const ret = getObject(arg0).node;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_now_64d0bb151e5d3889 = function() {
            const ret = Date.now();
            return ret;
        };
        imports.wbg.__wbg_origin_8c23d49bc1f609e9 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).origin;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments) };
        imports.wbg.__wbg_postMessage_586e33893b92051e = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).postMessage(getObject(arg1), getObject(arg2));
        }, arguments) };
        imports.wbg.__wbg_postMessage_6fd166b24db78adf = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).postMessage(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_postMessage_f648f854fd6c3f80 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).postMessage(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_process_5cff2739921be718 = function(arg0) {
            const ret = getObject(arg0).process;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_push_6edad0df4b546b2c = function(arg0, arg1) {
            const ret = getObject(arg0).push(getObject(arg1));
            return ret;
        };
        imports.wbg.__wbg_queueMicrotask_e410e98db024cc81 = function(arg0) {
            const ret = getObject(arg0).queueMicrotask;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_queueMicrotask_ec2456c3cd3e6990 = function(arg0) {
            queueMicrotask(getObject(arg0));
        };
        imports.wbg.__wbg_randomFillSync_d3c85af7e31cf1f8 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).randomFillSync(takeObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_read_4d173e86f707008c = function(arg0) {
            const ret = getObject(arg0).read();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_readable_5edeb421292684c3 = function(arg0) {
            const ret = getObject(arg0).readable;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_readable_c6b0eae3d758ef1e = function(arg0) {
            const ret = getObject(arg0).readable;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_readyState_236b61903e1dbb47 = function(arg0) {
            const ret = getObject(arg0).readyState;
            return ret;
        };
        imports.wbg.__wbg_ready_26e7f2af8ced9ce5 = function(arg0) {
            const ret = getObject(arg0).ready;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_ready_37d05ccdae17b91e = function(arg0) {
            const ret = getObject(arg0).ready;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_releaseLock_7878dddc005f738f = function(arg0) {
            getObject(arg0).releaseLock();
        };
        imports.wbg.__wbg_removeEventListener_6f4cfa6f356575bb = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
        }, arguments) };
        imports.wbg.__wbg_require_0c566c6f2eef6c79 = function() { return handleError(function () {
            const ret = module.require;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_resolve_0bf7c44d641804f9 = function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_send_c2b76ede40fcced1 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).send(getArrayU8FromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_setInterval_bede69d6c8f41bb4 = function() { return handleError(function (arg0, arg1) {
            const ret = setInterval(getObject(arg0), arg1);
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_set_23d69db4e5c66a6e = function(arg0, arg1, arg2) {
            getObject(arg0).set(getObject(arg1), arg2 >>> 0);
        };
        imports.wbg.__wbg_set_3807d5f0bfc24aa7 = function(arg0, arg1, arg2) {
            getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
        };
        imports.wbg.__wbg_set_4e647025551483bd = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_setbinaryType_3fa4a9e8d2cc506f = function(arg0, arg1) {
            getObject(arg0).binaryType = __wbindgen_enum_BinaryType[arg1];
        };
        imports.wbg.__wbg_setbitrate_ef69ae8e790b3a9a = function(arg0, arg1) {
            getObject(arg0).bitrate = arg1;
        };
        imports.wbg.__wbg_setcapture_b1387485d0f7947c = function(arg0, arg1) {
            getObject(arg0).capture = arg1 !== 0;
        };
        imports.wbg.__wbg_setcodec_27d37139d9ae2dfd = function(arg0, arg1, arg2) {
            getObject(arg0).codec = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setcodec_3436cfcfcc0831aa = function(arg0, arg1, arg2) {
            getObject(arg0).codec = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setcodec_4884924999c53174 = function(arg0, arg1, arg2) {
            getObject(arg0).codec = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setcodec_5a2a5aef66decae8 = function(arg0, arg1, arg2) {
            getObject(arg0).codec = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setdata_7a06f184fd37829c = function(arg0, arg1) {
            getObject(arg0).data = getObject(arg1);
        };
        imports.wbg.__wbg_setdata_c8315bc453f2189e = function(arg0, arg1) {
            getObject(arg0).data = getObject(arg1);
        };
        imports.wbg.__wbg_setduration_48fa96eb8e33fce0 = function(arg0, arg1) {
            getObject(arg0).duration = arg1;
        };
        imports.wbg.__wbg_setduration_ce5d5463202d86f0 = function(arg0, arg1) {
            getObject(arg0).duration = arg1;
        };
        imports.wbg.__wbg_seterror_3bbefc95e5ff3ffc = function(arg0, arg1) {
            getObject(arg0).error = getObject(arg1);
        };
        imports.wbg.__wbg_seterror_3f82054084efc4a4 = function(arg0, arg1) {
            getObject(arg0).error = getObject(arg1);
        };
        imports.wbg.__wbg_seterror_6268309d6702f374 = function(arg0, arg1) {
            getObject(arg0).error = getObject(arg1);
        };
        imports.wbg.__wbg_seterror_c16baf4ed2bfd51c = function(arg0, arg1) {
            getObject(arg0).error = getObject(arg1);
        };
        imports.wbg.__wbg_setheight_dcd9bb7792c70090 = function(arg0, arg1) {
            getObject(arg0).height = arg1 >>> 0;
        };
        imports.wbg.__wbg_setkeyframe_5ca11fa853c41254 = function(arg0, arg1) {
            getObject(arg0).keyFrame = arg1 !== 0;
        };
        imports.wbg.__wbg_setlatencymode_a3702e483dbe7d76 = function(arg0, arg1) {
            getObject(arg0).latencyMode = __wbindgen_enum_LatencyMode[arg1];
        };
        imports.wbg.__wbg_setnumberofchannels_877eaeadb01481c0 = function(arg0, arg1) {
            getObject(arg0).numberOfChannels = arg1 >>> 0;
        };
        imports.wbg.__wbg_setonce_87cf501e67ee47f7 = function(arg0, arg1) {
            getObject(arg0).once = arg1 !== 0;
        };
        imports.wbg.__wbg_setonmessage_4596c1308611382a = function(arg0, arg1) {
            getObject(arg0).onmessage = getObject(arg1);
        };
        imports.wbg.__wbg_setonmessage_876a303124885034 = function(arg0, arg1) {
            getObject(arg0).onmessage = getObject(arg1);
        };
        imports.wbg.__wbg_setoutput_ade8850ad35be202 = function(arg0, arg1) {
            getObject(arg0).output = getObject(arg1);
        };
        imports.wbg.__wbg_setoutput_b6889934ce55f3dc = function(arg0, arg1) {
            getObject(arg0).output = getObject(arg1);
        };
        imports.wbg.__wbg_setoutput_baf8ce12c24b631d = function(arg0, arg1) {
            getObject(arg0).output = getObject(arg1);
        };
        imports.wbg.__wbg_setoutput_e0617492edb59819 = function(arg0, arg1) {
            getObject(arg0).output = getObject(arg1);
        };
        imports.wbg.__wbg_setpassive_4e263ba5bfca97c5 = function(arg0, arg1) {
            getObject(arg0).passive = arg1 !== 0;
        };
        imports.wbg.__wbg_setreason_dd51f76e157a84ae = function(arg0, arg1, arg2) {
            getObject(arg0).reason = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setsamplerate_d6f816e5ec3e1212 = function(arg0, arg1) {
            getObject(arg0).sampleRate = arg1 >>> 0;
        };
        imports.wbg.__wbg_settimestamp_0f5bf59e7d3de24b = function(arg0, arg1) {
            getObject(arg0).timestamp = arg1;
        };
        imports.wbg.__wbg_settimestamp_13c0db66bc356b0c = function(arg0, arg1) {
            getObject(arg0).timestamp = arg1;
        };
        imports.wbg.__wbg_settype_1cbf3cb318ddac6b = function(arg0, arg1) {
            getObject(arg0).type = __wbindgen_enum_EncodedAudioChunkType[arg1];
        };
        imports.wbg.__wbg_settype_f6b737bc439b738e = function(arg0, arg1) {
            getObject(arg0).type = __wbindgen_enum_EncodedVideoChunkType[arg1];
        };
        imports.wbg.__wbg_settype_fd39465d237c2f36 = function(arg0, arg1, arg2) {
            getObject(arg0).type = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_setwidth_a1d982cc27e3594f = function(arg0, arg1) {
            getObject(arg0).width = arg1 >>> 0;
        };
        imports.wbg.__wbg_state_07b901e3b1a44515 = function(arg0) {
            const ret = getObject(arg0).state;
            return (__wbindgen_enum_CodecState.indexOf(ret) + 1 || 4) - 1;
        };
        imports.wbg.__wbg_state_12399e75f5363539 = function(arg0) {
            const ret = getObject(arg0).state;
            return (__wbindgen_enum_CodecState.indexOf(ret) + 1 || 4) - 1;
        };
        imports.wbg.__wbg_state_232d41bc33ec1e74 = function(arg0) {
            const ret = getObject(arg0).state;
            return (__wbindgen_enum_CodecState.indexOf(ret) + 1 || 4) - 1;
        };
        imports.wbg.__wbg_state_614d196c8f5828dc = function(arg0) {
            const ret = getObject(arg0).state;
            return (__wbindgen_enum_CodecState.indexOf(ret) + 1 || 4) - 1;
        };
        imports.wbg.__wbg_static_accessor_GLOBAL_0be7472e492ad3e3 = function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_GLOBAL_THIS_1a6eb482d12c9bfb = function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_SELF_1dc398a895c82351 = function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_static_accessor_WINDOW_ae1c80c7eea8d64a = function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_subarray_46adeb9b86949d12 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_then_0438fad860fe38e1 = function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_then_9d5d3795a1472eeb = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_timestamp_b0cdfe2c06749c63 = function(arg0) {
            const ret = getObject(arg0).timestamp;
            return ret;
        };
        imports.wbg.__wbg_timestamp_f8f8a06fc2e28212 = function(arg0) {
            const ret = getObject(arg0).timestamp;
            return ret;
        };
        imports.wbg.__wbg_toString_cbcf95f260c441ae = function(arg0) {
            const ret = getObject(arg0).toString();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_type_3b22d3d9a91b7f0d = function(arg0) {
            const ret = getObject(arg0).type;
            return (__wbindgen_enum_EncodedAudioChunkType.indexOf(ret) + 1 || 3) - 1;
        };
        imports.wbg.__wbg_type_95ca27ab94c48f13 = function(arg0) {
            const ret = getObject(arg0).type;
            return (__wbindgen_enum_EncodedVideoChunkType.indexOf(ret) + 1 || 3) - 1;
        };
        imports.wbg.__wbg_value_4c32fd138a88eee2 = function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_versions_a8e5a362e1f16442 = function(arg0) {
            const ret = getObject(arg0).versions;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_write_0aea81ae26043440 = function(arg0, arg1) {
            const ret = getObject(arg0).write(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_bigint_from_i64 = function(arg0) {
            const ret = arg0;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_bigint_from_u64 = function(arg0) {
            const ret = BigInt.asUintN(64, arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_bigint_get_as_i64 = function(arg0, arg1) {
            const v = getObject(arg1);
            const ret = typeof(v) === 'bigint' ? v : undefined;
            getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        };
        imports.wbg.__wbindgen_boolean_get = function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
            return ret;
        };
        imports.wbg.__wbindgen_cb_drop = function(arg0) {
            const obj = takeObject(arg0).original;
            if (obj.cnt-- == 1) {
                obj.a = 0;
                return true;
            }
            const ret = false;
            return ret;
        };
        imports.wbg.__wbindgen_closure_wrapper490 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 171, __wbg_adapter_50);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper492 = function(arg0, arg1, arg2) {
            const ret = makeClosure(arg0, arg1, 171, __wbg_adapter_53);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper494 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 171, __wbg_adapter_56);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper496 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 171, __wbg_adapter_56);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper498 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 171, __wbg_adapter_61);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper828 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 404, __wbg_adapter_64);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper917 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 455, __wbg_adapter_67);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper968 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 470, __wbg_adapter_70);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_in = function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        };
        imports.wbg.__wbindgen_is_bigint = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'bigint';
            return ret;
        };
        imports.wbg.__wbindgen_is_falsy = function(arg0) {
            const ret = !getObject(arg0);
            return ret;
        };
        imports.wbg.__wbindgen_is_function = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        };
        imports.wbg.__wbindgen_is_object = function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        };
        imports.wbg.__wbindgen_is_string = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'string';
            return ret;
        };
        imports.wbg.__wbindgen_is_undefined = function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        };
        imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
            const ret = getObject(arg0) === getObject(arg1);
            return ret;
        };
        imports.wbg.__wbindgen_jsval_loose_eq = function(arg0, arg1) {
            const ret = getObject(arg0) == getObject(arg1);
            return ret;
        };
        imports.wbg.__wbindgen_memory = function() {
            const ret = wasm.memory;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        };
        imports.wbg.__wbindgen_number_new = function(arg0) {
            const ret = arg0;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
            takeObject(arg0);
        };
        imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        };
        imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_throw = function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        };

        return imports;
    }

    function __wbg_init_memory(imports, memory) {

    }

    function __wbg_finalize_init(instance, module) {
        wasm = instance.exports;
        __wbg_init.__wbindgen_wasm_module = module;
        cachedDataViewMemory0 = null;
        cachedUint8ArrayMemory0 = null;


        wasm.__wbindgen_start();
        return wasm;
    }

    function initSync(module) {
        if (wasm !== undefined) return wasm;


        if (typeof module !== 'undefined') {
            if (Object.getPrototypeOf(module) === Object.prototype) {
                ({module} = module)
            } else {
                console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
            }
        }

        const imports = __wbg_get_imports();

        __wbg_init_memory(imports);

        if (!(module instanceof WebAssembly.Module)) {
            module = new WebAssembly.Module(module);
        }

        const instance = new WebAssembly.Instance(module, imports);

        return __wbg_finalize_init(instance, module);
    }

    async function __wbg_init(module_or_path) {
        if (wasm !== undefined) return wasm;


        if (typeof module_or_path !== 'undefined') {
            if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
                ({module_or_path} = module_or_path)
            } else {
                console.warn('using deprecated parameters for the initialization function; pass a single object instead')
            }
        }

        if (typeof module_or_path === 'undefined' && typeof script_src !== 'undefined') {
            module_or_path = script_src.replace(/\.js$/, '_bg.wasm');
        }
        const imports = __wbg_get_imports();

        if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
            module_or_path = fetch(module_or_path);
        }

        __wbg_init_memory(imports);

        const { instance, module } = await __wbg_load(await module_or_path, imports);

        return __wbg_finalize_init(instance, module);
    }

    wasm_bindgen = Object.assign(__wbg_init, { initSync }, __exports);

})();
