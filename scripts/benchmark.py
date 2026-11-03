#!/usr/bin/env python3
"""Reproducible local measurements. Never uses the application's real database."""
import argparse, concurrent.futures, json, os, pathlib, platform, resource, sqlite3, subprocess, threading, time, urllib.request

ROOT=pathlib.Path(__file__).resolve().parents[1]
BINARY=ROOT/'target/release/sorriprecos'

def memory(pid):
    try:
        lines=pathlib.Path(f'/proc/{pid}/status').read_text().splitlines()
        return int(next(x for x in lines if x.startswith('VmRSS:')).split()[1])*1024
    except (FileNotFoundError,StopIteration):return 0

def cpu(pid):
    fields=pathlib.Path(f'/proc/{pid}/stat').read_text().split()
    return (int(fields[13])+int(fields[14]))/os.sysconf('SC_CLK_TCK')

def run_import(command):
    started=time.perf_counter();p=subprocess.Popen(command,stdout=subprocess.DEVNULL)
    peak=0
    while p.poll() is None:
        peak=max(peak,memory(p.pid));time.sleep(.05)
    if p.returncode:raise RuntimeError(f'import failed: {p.returncode}')
    return {'seconds':round(time.perf_counter()-started,3),'peak_rss_bytes':peak}

def measure(base,requests,workers):
    queries=['arroz%205kg','arroz%20sintetico','variedade%200999','inexistente','arroz%200001']
    def request(i):
        started=time.perf_counter()
        with urllib.request.urlopen(base+'/v1/products/search?demo=true&q='+queries[i%len(queries)],timeout=30) as r:
            data=json.load(r)
            assert 'items' in data
        return (time.perf_counter()-started)*1000
    started=time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as executor:latencies=list(executor.map(request,range(requests)))
    elapsed=time.perf_counter()-started;latencies.sort()
    def percentile(p):return round(latencies[min(len(latencies)-1,int((len(latencies)-1)*p))],3)
    return dict(requests=requests,workers=workers,seconds=round(elapsed,3),rps=round(requests/elapsed,2),p50_ms=percentile(.5),p95_ms=percentile(.95),p99_ms=percentile(.99),errors=0)

def scenario(count,offers,requests,workers,port,directory):
    db=directory/f'{count}-{offers}.db'
    # Never overwrite an earlier result database.
    if db.exists():raise RuntimeError(f'{db} already exists; choose a new --output directory')
    prefix=[str(BINARY),'--database-url',f'sqlite://{db}']
    seed=run_import(prefix+['bench-seed','--products',str(count),'--offers',str(offers)])
    with sqlite3.connect(db) as con:con.execute('PRAGMA wal_checkpoint(TRUNCATE)')
    with open(directory/f'{count}-{offers}-api.log','w') as log:
        server=subprocess.Popen(prefix+['serve','--bind',f'127.0.0.1:{port}','--rate-limit','1000000'],stdout=log,stderr=log)
        base=f'http://127.0.0.1:{port}'
        try:
            for _ in range(100):
                if server.poll() is not None:raise RuntimeError('benchmark API failed to start')
                try:
                    urllib.request.urlopen(base+'/health',timeout=1).close();break
                except OSError:time.sleep(.1)
            measure(base,20,1)
            initial_cpu=cpu(server.pid);start=time.perf_counter()
            reading=measure(base,requests,workers)
            reading['cpu_percent_one_core']=round(100*(cpu(server.pid)-initial_cpu)/(time.perf_counter()-start),2)
            reading['rss_bytes']=memory(server.pid)
            # A second importer touches an existing subset while readers use the same WAL database.
            writer=subprocess.Popen(prefix+['bench-seed','--products',str(min(count,1000)),'--offers',str(offers)],stdout=subprocess.DEVNULL)
            try:
                mixed=measure(base,requests,workers)
                overlap=writer.poll() is None
                if writer.wait(timeout=120):raise RuntimeError('concurrent importer failed')
            finally:
                if writer.poll() is None:writer.terminate();writer.wait(timeout=5)
            mixed['writer_still_running_after_read_phase']=overlap
            with sqlite3.connect(db) as con:
                actual=con.execute('SELECT COUNT(*) FROM products').fetchone()[0]
                offer_count=con.execute('SELECT COUNT(*) FROM offers').fetchone()[0]
            return dict(products=actual,offers=offer_count,offers_per_product=offers,db_bytes=db.stat().st_size,import_result=seed,read=reading,concurrent_write=mixed)
        finally:
            server.terminate();server.wait(timeout=10)

def main():
    p=argparse.ArgumentParser();p.add_argument('--sizes',nargs='+',type=int,default=[10000,50000,100000]);p.add_argument('--offers',nargs='+',type=int,default=[1,3]);p.add_argument('--requests',type=int,default=400);p.add_argument('--workers',type=int,default=8);p.add_argument('--port',type=int,default=8099);p.add_argument('--output',type=pathlib.Path,default=ROOT/'artifacts/benchmarks')
    a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
    result={'environment':{'platform':platform.platform(),'cpu_count':os.cpu_count(),'python':platform.python_version(),'binary':str(BINARY),'profile':'release','date_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'cpu_model':next((x.split(':',1)[1].strip() for x in pathlib.Path('/proc/cpuinfo').read_text().splitlines() if x.startswith('model name')),'unknown')},'scenarios':[]}
    for count in a.sizes:
        for offers in a.offers:
            item=scenario(count,offers,a.requests,a.workers,a.port,a.output);result['scenarios'].append(item)
            (a.output/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(item),flush=True)
if __name__=='__main__':main()
