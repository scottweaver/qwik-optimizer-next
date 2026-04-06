//! Behavioral tests derived from the specification's Appendix B: Representative Examples.
//!
//! These 24 curated examples cover all 14 CONVs and are extracted directly from the
//! qwik-optimizer-spec.md Appendix B. Each test defines input code, expected output,
//! and the CONV transformations being verified.
//!
//! All tests start as `#[ignore]` -- they will be un-ignored progressively as the
//! transform implementation proceeds, following the same pattern as snapshot_tests.rs.
//!
//! CONV Coverage:
//! - CONV-01 Dollar Detection:     Examples 1, 2
//! - CONV-02 QRL Wrapping:         Examples 1, 2, 18, 19, 20
//! - CONV-03 Capture Analysis:     Examples 2, 3, 4, 22
//! - CONV-04 Props Destructuring:  Examples 4, 5
//! - CONV-05 Segment Extraction:   Examples 1, 6, 18, 24
//! - CONV-06 JSX Transform:        Examples 7, 8, 21
//! - CONV-07 Signal Optimization:  Examples 5, 9
//! - CONV-08 PURE Annotations:     Examples 1, 2, 10
//! - CONV-09 Dead Branch Elim:     Example 11
//! - CONV-10 Const Replacement:    Examples 12, 23
//! - CONV-11 Code Stripping:       Examples 13, 14
//! - CONV-12 Import Rewriting:     Examples 3, 15
//! - CONV-13 sync$ Serialization:  Example 16
//! - CONV-14 Noop QRL Handling:    Example 17

/// CONV coverage documentation: which spec examples cover which CONVs.
#[cfg(test)]
const CONV_COVERAGE: &[(&str, &[u8])] = &[
    ("CONV-01: Dollar Detection", &[1, 2]),
    ("CONV-02: QRL Wrapping", &[1, 2, 18, 19, 20]),
    ("CONV-03: Capture Analysis", &[2, 3, 4, 22]),
    ("CONV-04: Props Destructuring", &[4, 5]),
    ("CONV-05: Segment Extraction", &[1, 6, 18, 24]),
    ("CONV-06: JSX Transform", &[7, 8, 21]),
    ("CONV-07: Signal Optimization", &[5, 9]),
    ("CONV-08: PURE Annotations", &[1, 2, 10]),
    ("CONV-09: Dead Branch Elimination", &[11]),
    ("CONV-10: Const Replacement", &[12, 23]),
    ("CONV-11: Code Stripping", &[13, 14]),
    ("CONV-12: Import Rewriting", &[3, 15]),
    ("CONV-13: sync$ Serialization", &[16]),
    ("CONV-14: Noop QRL Handling", &[17]),
];

// ============================================================================
// CONV-01: Dollar Detection
// ============================================================================

#[cfg(test)]
mod conv_01_dollar_detection {
    use super::*;

    /// Example 1: Basic Dollar Detection and QRL Wrapping
    /// Snapshot: example_1
    /// Demonstrates: Three $() calls detected -- outer $(), nested onClick, component$()
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_01_basic_dollar_extraction() {
        let input = r#"
import { $, component, onRender } from '@qwik.dev/core';

export const renderHeader1 = $(() => {
  return (
    <div onClick={$((ctx) => console.log(ctx))}/>
  );
});
const renderHeader2 = component($(() => {
  console.log("mount");
  return render;
}));
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: 3 segments extracted (renderHeader1 body, onClick handler, renderHeader2 body)
        // Expected: root module replaces $() calls with qrl() imports
        // Expected: $ import removed, qrl import added in root
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }

    /// Example 2: Component with useStore and Capture Analysis
    /// Snapshot: example_functional_component
    /// Demonstrates: component$ -> componentQrl, import captures, captures: false
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_02_component_with_captures() {
        let input = r#"
import { $, component$, useStore } from '@qwik.dev/core';
const Header = component$(() => {
  const thing = useStore();
  const {foo, bar} = foo();

  return (
    <div>{thing}</div>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: component$ -> componentQrl in root
        // Expected: useStore re-imported in segment (import capture)
        // Expected: segment has captures: false (all vars declared inside)
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-03: Capture Analysis
// ============================================================================

#[cfg(test)]
mod conv_03_capture_analysis {
    use super::*;

    /// Example 3: Import Captures vs Self-Imports
    /// Snapshot: example_capture_imports
    /// Demonstrates: CSS imports re-imported in segments, not captured via _captures
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_03_import_captures_vs_self_imports() {
        let input = r#"
import { component$, useStyles$ } from '@qwik.dev/core';
import css1 from './global.css';
import css2 from './style.css';
import css3 from './style.css';

export const App = component$(() => {
  useStyles$(`${css1}${css2}`);
  useStyles$(css3);
})
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: css1, css2, css3 are import captures (re-imported in segments)
        // Expected: useStyles$ -> useStylesQrl
        // Expected: template literal becomes segment export value
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }

    /// Example 4: Multiple Captured Variables
    /// Snapshot: example_multi_capture
    /// Demonstrates: Props capture as _rawProps, constant inlining, .w() capture passing
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_04_multiple_captured_variables() {
        let input = r#"
import { $, component$ } from '@qwik.dev/core';

export const Foo = component$(({foo}) => {
  const arg0 = 20;
  return $(() => {
    const fn = ({aaa}) => aaa;
    return (
      <div>
        {foo}{fn()}{arg0}
      </div>
    )
  });
})

export const Bar = component$(({bar}) => {
  return $(() => {
    return (
      <div>
        {bar}
      </div>
    )
  });
})
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: ({foo}) -> (_rawProps) in parent segment
        // Expected: inner segment captures _rawProps, accesses as _rawProps.foo
        // Expected: arg0 (constant 20) inlined directly, not captured
        // Expected: .w([_rawProps]) on QRL to pass capture
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-04: Props Destructuring
// ============================================================================

#[cfg(test)]
mod conv_04_props_destructuring {
    use super::*;

    /// Example 5: Props Destructuring with Colon Syntax
    /// Snapshot: destructure_args_colon_props
    /// Demonstrates: bind:value -> _wrapProp, Fragment import for <>
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_05_props_colon_syntax() {
        let input = r#"
import { component$ } from "@qwik.dev/core";
export default component$((props) => {
  const { 'bind:value': bindValue } = props;
  return (
    <>
    {bindValue}
    </>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: props passes through as-is (not destructured in params)
        // Expected: _wrapProp(props, "bind:value") for signal wrapping
        // Expected: Fragment import added for <> syntax
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-05: Segment Extraction
// ============================================================================

#[cfg(test)]
mod conv_05_segment_extraction {
    use super::*;

    /// Example 6: Variable Migration into Segments
    /// Snapshot: example_segment_variable_migration
    /// Demonstrates: helperFn migrated, SHARED_CONFIG stays with _auto_ re-export
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_06_variable_migration() {
        let input = r#"
import { component$ } from '@qwik.dev/core';

const helperFn = (msg) => {
  console.log('Helper: ' + msg);
  return msg.toUpperCase();
};

const SHARED_CONFIG = { value: 42 };

export const publicHelper = () => console.log('public');

export const App = component$(() => {
  const result = helperFn('hello');
  return <div>{result} {SHARED_CONFIG.value}</div>;
});

export const Other = component$(() => {
  return <div>{SHARED_CONFIG.value}</div>;
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: helperFn moved entirely into App segment (only used there)
        // Expected: SHARED_CONFIG stays in root, re-exported as _auto_SHARED_CONFIG
        // Expected: segments import via _auto_ prefix self-import
        // Expected: publicHelper remains in root (it's an export)
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-06: JSX Transform
// ============================================================================

#[cfg(test)]
mod conv_06_jsx_transform {
    use super::*;

    /// Example 7: JSX Transform Basics
    /// Snapshot: example_jsx
    /// Demonstrates: _jsxSorted/_jsxSplit, static/dynamic prop split, Fragment, spread
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_07_jsx_basics() {
        let input = r#"
import { $, component$, h, Fragment } from '@qwik.dev/core';

export const Lightweight = (props) => {
  return (
    <div>
      <>
        <div/>
        <button {...props}/>
      </>
    </div>
  )
};
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: _jsxSorted for normal elements, _jsxSplit for spread
        // Expected: Fragment for <>, _getVarProps/_getConstProps for spread
        // Expected: flags 3 for immutable empty elements
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }

    /// Example 8: Event Handler JSX Transforms
    /// Snapshot: example_jsx_listeners
    /// Demonstrates: onClick$ -> q-e:click, event handler extraction, host: prefix
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_08_event_handler_jsx() {
        let input = r#"
import { $, component$ } from '@qwik.dev/core';

export const Foo = component$(() => {
  return $(() => {
    const handler = $(() => console.log('reused'));
    return (
      <div
        onClick$={()=>console.log('onClick$')}
        onDocumentScroll$={()=>console.log('onDocumentScroll')}
        on-cLick$={()=>console.log('on-cLick$')}
        host:onClick$={()=>console.log('host:onClick$')}
        onKeyup$={handler}
        onDocument:keyup$={handler}
        onWindow:keyup$={handler}
        custom$={()=>console.log('custom')}
      />
    )
  });
}, { tagName: "my-foo" });
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: onClick$ -> q-e:click (lowercased, on stripped)
        // Expected: onDocumentScroll$ -> q-e:documentscroll
        // Expected: on-cLick$ -> q-e:c-lick (hyphen preserved)
        // Expected: host: prefix preserved as-is
        // Expected: custom$ stays as custom$ (no on prefix)
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }

    /// Example 21: bind:value and bind:checked Sugar
    /// Snapshot: example_input_bind
    /// Demonstrates: bind:value -> value prop + _val handler, bind:checked -> _chk
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_21_bind_sugar() {
        let input = r#"
import { component$, $ } from '@qwik.dev/core';

export const Greeter = component$(() => {
  const value = useSignal(0);
  const checked = useSignal(false);
  const stuff = useSignal();
  return (
    <>
      <input bind:value={value} />
      <input bind:checked={checked} />
      <input bind:stuff={stuff} />
      <div>{value}</div>
      <div>{value.value}</div>
    </>
  )
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "inline",
            mode: "lib",
            ..Default::default()
        };

        // Expected: bind:value -> value prop + q-e:input with _val handler
        // Expected: bind:checked -> checked prop + q-e:input with _chk handler
        // Expected: bind:stuff passes through as-is (not value or checked)
        // Expected: {value} passes signal directly, {value.value} -> _wrapProp
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-07: Signal Optimization
// ============================================================================

#[cfg(test)]
mod conv_07_signal_optimization {
    use super::*;

    /// Example 9: Signal Optimization
    /// Snapshot: example_derived_signals_cmp
    /// Demonstrates: _wrapProp, _fnSignal, optimizable vs non-optimizable
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_09_signal_optimization() {
        let input = r#"
import { component$, useStore, mutable } from '@qwik.dev/core';
import {dep} from './file';
import {Cmp} from './cmp';

export const App = component$(() => {
  const signal = useSignal(0);
  const store = useStore({});
  return (
    <Cmp
      staticText="text"
      staticNumber={1}
      signal={signal}
      signalValue={signal.value}
      signalComputedValue={12 + signal.value}
      store={store.address.city.name}
      storeComputed={store.address.city.name ? 'true' : 'false'}
      dep={dep}
      depAccess={dep.thing}
      noInline={signal.value()}
      noInline2={signal.value + unknown()}
      noInline3={mutable(signal)}
      noInline4={signal.value + dep}
    />
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "inline",
            mode: "lib",
            ..Default::default()
        };

        // Expected: signal.value -> _wrapProp(signal)
        // Expected: 12 + signal.value -> _fnSignal(_hf0, [signal], _hf0_str)
        // Expected: store.address.city.name -> _fnSignal
        // Expected: signal.value() NOT optimizable (method call)
        // Expected: signal.value + unknown() NOT optimizable (function call)
        // Expected: mutable(signal) NOT optimizable
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-08: PURE Annotations
// ============================================================================

#[cfg(test)]
mod conv_08_pure_annotations {
    use super::*;

    /// Example 10: PURE Annotation on componentQrl
    /// Snapshot: example_functional_component_2
    /// Demonstrates: PURE on qrl() and componentQrl(), constant inlining, captures
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_10_pure_annotations() {
        let input = r#"
import { $, component$, useStore } from '@qwik.dev/core';
export const useCounter = () => {
  return useStore({count: 0});
}

export const STEP = 1;

export const App = component$((props) => {
  const state = useCounter();
  const thing = useStore({thing: 0});
  const STEP_2 = 2;
  const count2 = state.count * 2;
  return (
    <div onClick$={() => state.count+=count2 }>
      <span>{state.count}</span>
      {buttons.map(btn => (
        <button
          onClick$={() => state.count += btn.offset + thing + STEP + STEP_2 + props.step}
        >
          {btn.name}
        </button>
      ))}
    </div>
  );
})
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: /*#__PURE__*/ on both qrl() and componentQrl()
        // Expected: STEP imported (module-level export), STEP_2 inlined as 2
        // Expected: button handler captures props, state, thing via _captures
        // Expected: btn comes from .map() loop context (q:p binding)
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-09: Dead Branch Elimination
// ============================================================================

#[cfg(test)]
mod conv_09_dead_branch_elimination {
    use super::*;

    /// Example 11: Dead Branch Elimination
    /// Snapshot: example_dead_code
    /// Demonstrates: if(false) removed, unused import dropped, empty callback
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_11_dead_branch_elimination() {
        let input = r#"
import { component$ } from '@qwik.dev/core';
import { deps } from 'deps';

export const Foo = component$(({foo}) => {
  useMount$(() => {
    if (false) {
      deps();
    }
  });
  return (
    <div />
  );
})
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: if(false) block entirely eliminated
        // Expected: deps import removed (no longer referenced)
        // Expected: useMount$ callback body becomes empty ()=>{}
        // Expected: ({foo}) -> (_rawProps) even though foo unused
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-10: Const Replacement
// ============================================================================

#[cfg(test)]
mod conv_10_const_replacement {
    use super::*;

    /// Example 12: isServer/isBrowser Const Replacement
    /// Snapshot: example_build_server
    /// Demonstrates: isServer -> true, isBrowser -> false in server mode
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_12_const_replacement_server() {
        let input = r#"
import { component$, useStore, isDev, isServer as isServer2 } from '@qwik.dev/core';
import { isServer, isBrowser as isb } from '@qwik.dev/core/build';
import { mongodb } from 'mondodb';
import { threejs } from 'threejs';
import L from 'leaflet';

export const functionThatNeedsWindow = () => {
  if (isb) {
    console.log('l', L);
    window.alert('hey');
  }
};

export const App = component$(() => {
  useMount$(() => {
    if (isServer) {
      console.log('server', mongodb());
    }
    if (isb) {
      console.log('browser', new threejs());
    }
  });
  return (
    <Cmp>
      {isServer2 && <p>server</p>}
      {isb && <p>server</p>}
    </Cmp>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "prod",
            is_server: Some(true),
            ..Default::default()
        };

        // Expected: isServer -> true, isBrowser/isb -> false
        // Expected: functionThatNeedsWindow body eliminated (isb is false)
        // Expected: leaflet, threejs imports removed (dead code)
        // Expected: mongodb import preserved (server path survives)
        // Expected: prod mode short s_ prefix for segment names
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }

    /// Example 23: Lib Mode -- No Const Replacement
    /// Snapshot: example_lib_mode
    /// Demonstrates: isServer/isBrowser NOT replaced in lib mode
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_23_lib_mode_no_const_replace() {
        let input = r#"
import { $, component$, server$, useStyle$, useTask$, useSignal } from '@qwik.dev/core';

export const Works = component$((props) => {
  useStyle$(STYLES);
  const text = 'hola';
  const sig = useSignal('hola');
  useTask$(() => {
    console.log(sig.value, text);
  });
  return (
    <div onClick$={server$(() => console.log('in server', sig.value, text))}></div>
  );
});

const STYLES = '.class {}';
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "inline",
            mode: "lib",
            ..Default::default()
        };

        // Expected: No const replacement in lib mode
        // Expected: server$ -> serverQrl with inlined body
        // Expected: text constant ('hola') inlined directly in segments
        // Expected: sig captured via _captures in inlined QRLs
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-11: Code Stripping
// ============================================================================

#[cfg(test)]
mod conv_11_code_stripping {
    use super::*;

    /// Example 13: Code Stripping -- strip_exports
    /// Snapshot: example_strip_client_code
    /// Demonstrates: strip_exports replaces segment bodies with null
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_13_strip_exports() {
        let input = r#"
import { component$, useClientMount$, useStore, useTask$ } from '@qwik.dev/core';
import mongo from 'mongodb';
import redis from 'redis';
import threejs from 'threejs';
import { a } from './keep';
import { b } from '../keep2';
import { c } from '../../remove';

export const Parent = component$(() => {
  const state = useStore({ text: '' });

  useClientMount$(async () => {
    state.text = await mongo.users();
    redis.set(state.text, a, b, c);
  });

  useTask$(() => {
    // Code
  });

  return (
    <div
      shouldRemove$={() => state.text}
      onClick$={() => console.log('parent', state, threejs)}
    >
      <Div
        onClick$={() => console.log('keep')}
        render$={() => state.text}
      />
      {state.text}
    </div>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "inline",
            mode: "lib",
            strip_exports: Some(vec!["useClientMount$".to_string()]),
            ..Default::default()
        };

        // Expected: useClientMount$ segment body -> null
        // Expected: shouldRemove$ and onClick$ segments -> null
        // Expected: mongo, redis, threejs imports dropped
        // Expected: useTask$, Div onClick$ preserved
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }

    /// Example 14: Server Code Stripping
    /// Snapshot: example_strip_server_code
    /// Demonstrates: strip_ctx_name, nested $() preserved inside stripped function
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_14_server_code_stripping() {
        let input = r#"
import { component$, serverLoader$, serverStuff$, $, client$, useStore, useTask$ } from '@qwik.dev/core';
import { isServer } from '@qwik.dev/core';
import mongo from 'mongodb';
import redis from 'redis';
import { handler } from 'serverless';

export const Parent = component$(() => {
  const state = useStore({ text: '' });

  useTask$(async () => {
    if (!isServer) return;
    state.text = await mongo.users();
    redis.set(state.text);
  });

  serverStuff$(async () => {
    const a = $(() => { /* from $(), should not be removed */ });
    const b = client$(() => { /* from client$(), should not be removed */ });
    return [a,b];
  })

  serverLoader$(handler);

  useTask$(() => { /* Code */ });

  return (
    <div onClick$={() => console.log('parent')}>
      {state.text}
    </div>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            strip_ctx_name: Some(vec![
                "serverLoader$".to_string(),
                "serverStuff$".to_string(),
            ]),
            ..Default::default()
        };

        // Expected: serverStuff$ and serverLoader$ segment bodies -> null
        // Expected: $() and client$() INSIDE serverStuff$ are preserved
        // Expected: useTask$ segments preserved
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-12: Import Rewriting
// ============================================================================

#[cfg(test)]
mod conv_12_import_rewriting {
    use super::*;

    /// Example 15: Legacy Import Rewriting
    /// Snapshot: rename_builder_io
    /// Demonstrates: @builder.io/qwik -> @qwik.dev/core, qwik-city -> router
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_15_legacy_import_rewriting() {
        let input = r#"
import { $, component$ } from "@builder.io/qwik";
import { isDev } from "@builder.io/qwik/build";
import { stuff } from "@builder.io/qwik-city";
import { moreStuff } from "@builder.io/qwik-city/more/here";
import { qwikify$ } from "@builder.io/qwik-react";
import sdk from "@builder.io/sdk";

export const Foo = qwikify$(MyReactComponent);
export const Bar = $("a thing");
export const App = component$(() => {
  sdk.hello();
  if (isDev) { stuff() } else { moreStuff() }
  return "hi";
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: @builder.io/qwik -> @qwik.dev/core
        // Expected: @builder.io/qwik/build -> @qwik.dev/core/build
        // Expected: @builder.io/qwik-city -> @qwik.dev/router
        // Expected: @builder.io/qwik-city/more/here -> @qwik.dev/router/more/here
        // Expected: @builder.io/qwik-react -> @qwik.dev/react
        // Expected: @builder.io/sdk NOT rewritten (not a Qwik package)
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-13: sync$ Serialization
// ============================================================================

#[cfg(test)]
mod conv_13_sync_serialization {
    use super::*;

    /// Example 16: sync$ Serialization
    /// Snapshot: example_of_synchronous_qrl
    /// Demonstrates: sync$ -> _qrlSync with function + minified string
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_16_sync_serialization() {
        let input = r#"
import { sync$, component$ } from "@qwik.dev/core";

export default component$(() => {
  return (
    <>
      <input onClick$={sync$(function(event, target) {
        // comment should be removed
        event.preventDefault();
      })}/>
      <input onClick$={sync$((event, target) => {
        event.preventDefault();
      })}/>
      <input onClick$={sync$((event, target) => event.preventDefault())}/>
    </>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: sync$ -> _qrlSync(fn, minifiedString)
        // Expected: comments stripped from minified string
        // Expected: both function declarations and arrow functions supported
        // Expected: sync$ handlers stay in varProps
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// CONV-14: Noop QRL Handling
// ============================================================================

#[cfg(test)]
mod conv_14_noop_qrl {
    use super::*;

    /// Example 17: Noop QRL Handling -- _noopQrlDEV
    /// Snapshot: example_noop_dev_mode
    /// Demonstrates: dev mode qrlDEV, stripped segments as _noopQrlDEV, JSX debug info
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_17_noop_qrl_dev() {
        let input = r#"
import { component$, useStore, serverStuff$, $ } from '@qwik.dev/core';

export const App = component$(() => {
  const stuff = useStore();
  serverStuff$(async () => {
    console.log(stuff.count)
  })
  serverStuff$(async () => {
    // should be removed
  })

  return (
    <Cmp>
      <p class="stuff"
        shouldRemove$={() => stuff.count}
        onClick$={() => console.log('warn')}
      >
        Hello Qwik
      </p>
    </Cmp>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "inline",
            mode: "dev",
            is_server: Some(true),
            strip_ctx_name: Some(vec!["serverStuff$".to_string()]),
            ..Default::default()
        };

        // Expected: qrl() -> qrlDEV() with debug metadata (file, lo, hi, displayName)
        // Expected: stripped segments -> _noopQrlDEV with lo:0, hi:0
        // Expected: JSX gets fileName, lineNumber, columnNumber debug info
        // Expected: sentinel 4294901760 (0xFFFF0000) in noop QRL names
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// Entry Strategy & Mode Variants
// ============================================================================

#[cfg(test)]
mod conv_02_entry_strategy_modes {
    use super::*;

    /// Example 18: Inline Entry Strategy
    /// Snapshot: example_inlined_entry_strategy
    /// Demonstrates: _noopQrl + .s() inline pattern, no separate segment files
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_18_inline_entry_strategy() {
        let input = r#"
import { component$, useBrowserVisibleTask$, useStore, useStyles$ } from '@qwik.dev/core';
import { thing } from './sibling';
import mongodb from 'mongodb';

export const Child = component$(() => {
  useStyles$('somestring');
  const state = useStore({ count: 0 });
  useBrowserVisibleTask$(() => {
    state.count = thing.doStuff() + import("./sibling");
  });
  return (
    <div onClick$={() => console.log(mongodb)}>
    </div>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "inline",
            mode: "lib",
            ..Default::default()
        };

        // Expected: all segments inlined in single file
        // Expected: _noopQrl("symbolName") + .s(fn) pattern
        // Expected: no separate segment files generated
        // Expected: _captures imported at module level
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }

    /// Example 19: Dev Mode QRL Variants
    /// Snapshot: example_dev_mode
    /// Demonstrates: qrl -> qrlDEV with file/lo/hi/displayName metadata
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_19_dev_mode_qrl() {
        let input = r#"
import { component$, useStore } from '@qwik.dev/core';

export const App = component$(() => {
  return (
    <Cmp>
      <p class="stuff" onClick$={() => console.log('warn')}>Hello Qwik</p>
    </Cmp>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "dev",
            ..Default::default()
        };

        // Expected: qrl -> qrlDEV with {file, lo, hi, displayName}
        // Expected: JSX elements get {fileName, lineNumber, columnNumber}
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }

    /// Example 20: Prod Mode Short Names
    /// Snapshot: example_prod_node
    /// Demonstrates: s_ prefix + hash for symbol names in prod mode
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_20_prod_mode_short_names() {
        let input = r#"
import { component$ } from '@qwik.dev/core';

export const Foo = component$(() => {
  return (
    <div>
      <div onClick$={() => console.log('first')}/>
      <div onClick$={() => console.log('second')}/>
      <div onClick$={() => console.log('third')}/>
    </div>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "prod",
            ..Default::default()
        };

        // Expected: symbol names use s_ prefix + hash (e.g., s_HTDRsvUbLiE)
        // Expected: import paths still use full descriptive filenames
        // Expected: local vars use q_s_ prefix
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// Loop & Edge Cases
// ============================================================================

#[cfg(test)]
mod conv_03_loop_captures {
    use super::*;

    /// Example 22: Loop Capture Edge Case
    /// Snapshot: should_transform_nested_loops
    /// Demonstrates: nested loop captures, q:p binding, per-iteration .w()
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_22_nested_loop_captures() {
        let input = r#"
import { component$, useSignal, Signal } from '@qwik.dev/core';
const Foo = component$(function() {
  const data = useSignal<Signal<any>[]>([]);
  const data2 = useSignal<Signal<any>[]>([]);
  return <div>
    {data.value.map(row => (
      <div onClick$={() => console.log(row.value.id)}>
        {data2.value.map(item => (
          <p onClick$={() => console.log(row.value.id, item.value.id)}>
            {row.value.id}-{item.value.id}
          </p>
        ))}
      </div>
    ))}
  </div>;
})
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "Segment",
            mode: "lib",
            ..Default::default()
        };

        // Expected: outer handler gets row via q:p binding (3rd param)
        // Expected: inner handler captures row via _captures, gets item via q:p
        // Expected: per-iteration .w([row]) creates scoped QRL instances
        // Expected: _fnSignal for row.value.id and item.value.id text interpolation
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// Config Effects
// ============================================================================

#[cfg(test)]
mod conv_05_config_effects {
    use super::*;

    /// Example 24: preserve_filenames Config Effect
    /// Snapshot: example_preserve_filenames
    /// Demonstrates: full descriptive names preserved even in prod-like scenarios
    #[test]
    #[ignore = "Transform not yet implemented"]
    fn test_spec_example_24_preserve_filenames() {
        let input = r#"
import { component$, useStore } from '@qwik.dev/core';

export const App = component$((props) => {
  return (
    <Cmp>
      <p class="stuff" onClick$={() => console.log('warn')}>Hello Qwik</p>
    </Cmp>
  );
});
"#;

        let _config = SpecExampleConfig {
            entry_strategy: "inline",
            mode: "lib",
            preserve_filenames: true,
            ..Default::default()
        };

        // Expected: segment names use full descriptive names, not s_ prefix
        // Expected: _noopQrl symbol names use full descriptive form
        let _ = input;
        todo!("Wire to transform_modules() when implemented");
    }
}

// ============================================================================
// Test configuration helper
// ============================================================================

/// Simplified configuration for spec examples.
/// Maps to TransformModulesOptions when the transform is implemented.
#[derive(Debug)]
#[allow(dead_code)]
struct SpecExampleConfig {
    entry_strategy: &'static str,
    mode: &'static str,
    is_server: Option<bool>,
    strip_exports: Option<Vec<String>>,
    strip_ctx_name: Option<Vec<String>>,
    preserve_filenames: bool,
}

impl Default for SpecExampleConfig {
    fn default() -> Self {
        Self {
            entry_strategy: "Segment",
            mode: "lib",
            is_server: None,
            strip_exports: None,
            strip_ctx_name: None,
            preserve_filenames: false,
        }
    }
}
