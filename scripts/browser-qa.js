// Run this inside the collaborative preview's page context, while signed into a
// living demo crewmate. It operates the real activity controls against the server.
// Results remain available as window.__uiQa. This is developer QA, not app code.
(() => {
  const qa = window.__uiQa = { status: 'running', stage: '', passes: [], error: null, wire_drag_visible: false };
  const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
  const token = () => sessionStorage.getItem('impostor-token') || localStorage.getItem('impostor-last-token');
  async function waitFor(fn, timeout = 15000) {
    const start = Date.now();
    while (Date.now() - start < timeout) { const result = fn(); if (result) return result; await sleep(40); }
    throw new Error(`Timeout at ${qa.stage}`);
  }
  async function command(command) {
    const result = await (await fetch('/api/command', { method: 'POST', headers: {'Content-Type':'application/json'}, body:JSON.stringify({token:token(),command}) })).json();
    if (!result.ok) throw new Error(result.error); return result;
  }
  async function tap(selector) {
    const element = await waitFor(() => document.querySelector(selector));
    element.click(); await sleep(25);
  }
  const setRange = (selector,value) => { const el=document.querySelector(selector); if(!el)throw new Error(`Missing control ${selector}`);Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(el,String(value));el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true})); };
  async function run() {
    if (document.querySelector('.activity-modal')) {
      document.querySelector('[aria-label="Close activity"]').click(); await sleep(120);
    }
    for (const kind of ['wires','code','signal','battery','memory','anomaly','circuits','password','mixing','transfer','fault','frequency','breaker','timing','route_power','switches','cargo']) {
      qa.stage = kind;
      const result = await command({type:'start_task',station_id:kind,interaction:'qr'});
      const c = result.state.active_task.challenge;
      await waitFor(() => document.querySelector('.activity-modal'));
      if (kind === 'wires') {
        const board = document.querySelector('.wire-board');
        board.scrollIntoView({block:'center'}); await sleep(100);
        for (let left = 0; left < 4; left++) {
          const source = document.querySelector(`[data-left="${left}"]`);
          const target = document.querySelector(`[data-right="${c.right_order.indexOf(left)}"]`);
          const a = source.getBoundingClientRect(), b = target.getBoundingClientRect();
          const capture = source.setPointerCapture;
          // Synthetic browser events cannot acquire a real OS pointer. The live
          // handlers and geometry still run; native pointer capture is manual QA.
          source.setPointerCapture = () => {};
          source.dispatchEvent(new PointerEvent('pointerdown', {bubbles:true,isPrimary:true,pointerId:44,button:0,clientX:a.x+a.width/2,clientY:a.y+a.height/2}));
          await sleep(30);
          board.dispatchEvent(new PointerEvent('pointermove', {bubbles:true,isPrimary:true,pointerId:44,buttons:1,clientX:b.x+b.width/2-35,clientY:b.y+b.height/2}));
          await sleep(35);
          if (!document.querySelector('.loose-wire path')) throw new Error('The dragged cable is not visible');
          qa.wire_drag_visible = true;
          board.dispatchEvent(new PointerEvent('pointerup', {bubbles:true,isPrimary:true,pointerId:44,button:0,clientX:b.x+b.width/2,clientY:b.y+b.height/2}));
          source.setPointerCapture = capture; await sleep(60);
        }
      } else if (kind === 'code' || kind === 'password') {
        await waitFor(() => document.querySelector('[aria-label="Digit 1"]'));
        for (const digit of String(kind === 'code' ? c.code : c.answer)) await tap(`[aria-label="Digit ${digit}"]`);
        await tap('[aria-label="Submit code"]');
      } else if (kind === 'signal') {
        const range = document.querySelector('[aria-label="Signal frequency"]');
        Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(range,String(c.target));
        range.dispatchEvent(new Event('input',{bubbles:true})); range.dispatchEvent(new Event('change',{bubbles:true}));
      } else if (kind === 'battery') {
        const start = performance.now();
        await new Promise((resolve,reject) => {
          function tick() {
            if (document.querySelector('.task-success')) return resolve();
            if (performance.now()-start > 20000) return reject(new Error('Battery capture did not finish'));
            const marker = document.querySelector('.timing-track>i');
            const position = parseFloat(marker?.style.left || '-1');
            if (position > c.zone_start+4 && position < c.zone_end-4) {
              [...document.querySelectorAll('.activity-modal button')].find(b=>b.textContent.includes('Capture charge'))?.click();
            }
            requestAnimationFrame(tick);
          }
          requestAnimationFrame(tick);
        });
      } else if (kind === 'memory') {
        await waitFor(() => { const button = document.querySelector('[aria-label="Memory pad 1"]'); return button && !button.disabled; });
        for (const pad of c.sequence) await tap(`[aria-label="Memory pad ${pad+1}"]`);
      } else if (kind === 'anomaly') {
        await tap(`[aria-label="Symbol ${c.odd_index+1}"]`);
      } else if (kind === 'mixing') {
        setRange('#mix-ratio',c.target_a); await sleep(40); setRange('#mix-b',c.target_b);
      } else if (kind === 'frequency') {
        setRange('#frequency-tuner',c.target);
      } else if (kind === 'fault') {
        await tap(`.fault-grid button:nth-child(${c.fault_index+1})`);
      } else if (kind === 'breaker') {
        for(const d of c.code)await tap(`[aria-label="Digit ${d}"]`);
        await tap('[aria-label="Test code"]');
      } else if (kind === 'switches') {
        for(const n of c.order)await tap(`[aria-label="Switch ${n+1}"]`);
      } else if (kind === 'cargo') {
        for(const item of c.items) await tap(`[aria-label="Sort ${item.label} into ${c.categories[item.category]}"]`);
      } else if (kind === 'transfer') {
        const tracking = setInterval(()=>{const receiver=document.querySelector('.receiver');if(receiver)setRange('#transfer-cursor',parseFloat(receiver.style.left)+parseFloat(receiver.style.width)/2);},30);
        try{await waitFor(()=>document.querySelector('.task-success'),20000);}finally{clearInterval(tracking);}
      } else if (kind === 'timing') {
        const tracking=setInterval(()=>{const marker=document.querySelector('.timing-track>i');const pos=parseFloat(marker?.style.left||'-1');if(pos>c.zone_start+1 && pos<c.zone_end-1)document.querySelector('[aria-label="Capture timing marker"]')?.click();},10);
        try{await waitFor(()=>document.querySelector('.task-success'),20000);}finally{clearInterval(tracking);}
      } else if (kind === 'circuits'  || kind === 'route_power') {
        for (let index=0;index<9;index++) {
          const turns = (c.solution[index]-c.rotations[index]+4)%4;
          for (let turn=0;turn<turns;turn++) await tap(`[aria-label="Rotate circuit tile ${index+1}"]`);
        }
      }
      await waitFor(() => document.querySelector('.task-success'));
      qa.passes.push(kind);
      [...document.querySelectorAll('.activity-modal button')].find(b=>b.textContent.includes('Back to mission')).click();
      await waitFor(() => !document.querySelector('.activity-modal'));
      await sleep(80);
    }
    qa.status = 'passed'; qa.stage = 'complete';
  }
  run().catch(error => { qa.status='failed';qa.error=error.message; });
  return 'Activity UI checks started. Inspect window.__uiQa for progress.';
})();
