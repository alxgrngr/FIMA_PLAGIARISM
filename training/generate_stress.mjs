// Produce an AI-only stress set from public prompts without sending data to an API.
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const [install, weights, source, out, promptFile] = process.argv.slice(2);
if (!install || !weights || !source || !out || fs.existsSync(out)) {
    throw new Error('Usage: node training/generate_stress.mjs INSTALL_DIR MODEL.gguf MODEL_SOURCE NEW_OUTPUT.jsonl [PROMPTS.json]');
}

const prompts = [
    'Uraikan hubungan pembagian giliran kerja dengan konflik antaranggota kelompok produksi.',
    'Jelaskan alasan anggota tim enggan menyampaikan temuan saat briefing keselamatan.',
    'Tuliskan pembahasan hasil observasi kelompok tentang keputusan supervisor ketika target produksi terlambat.',
    'Bandingkan fungsi penghargaan individu dan penghargaan kelompok pada tim pengendali mutu.',
    'Tuliskan refleksi praktikum tentang pembagian peran yang timpang pada simulasi lini perakitan.',
    'Jelaskan peran anggota yang bertindak sebagai penghubung tim gudang dan tim perencanaan.',
    'Tulis interpretasi tentang hubungan beban kerja dan komunikasi saat pergantian shift.',
    'Tulis saran bagi kepala bagian agar tingkat absensi rapat koordinasi turun.',
    'Jelaskan dampak kepemimpinan partisipatif ketika operator berbeda pendapat tentang standar kerja.',
    'Tulis latar belakang penilaian keadilan organisasi pada pembagian tugas pemeliharaan mesin.',
    'Ringkas alasan tim gagal mengikuti prosedur penyelesaian masalah dalam simulasi kelompok.',
    'Tulis pembahasan mengenai cara kerja tim merespons umpan balik dari asisten praktikum.',
];
const samples = promptFile ? JSON.parse(fs.readFileSync(promptFile, 'utf8')) :
    prompts.map((prompt, index) => ({group: 'org-behavior-prompt-' + (index + 1), prompt}));
if (!Array.isArray(samples) || samples.length === 0 || samples.some(sample =>
    typeof sample?.group !== 'string' || !sample.group.trim() ||
    typeof sample.prompt !== 'string' || !sample.prompt.trim()
)) {
    throw new Error('Each prompt needs a group and prompt string.');
}
const entry = path.resolve(install, 'node_modules/node-llama-cpp/dist/index.js');
const { getLlama, LlamaChatSession, LlamaLogLevel } = await import(pathToFileURL(entry));
const llama = await getLlama({gpu: false, logLevel: LlamaLogLevel.error});
const generated = [];

try {
    const model = await llama.loadModel({modelPath: weights});
    try {
        for (const [index, {group, prompt}] of samples.entries()) {
            const context = await model.createContext({contextSize: 1024, threads: 2});
            try {
                const session = new LlamaChatSession({
                    contextSequence: context.getSequence(),
                    systemPrompt: 'Anda menulis laporan praktikum Teknik Industri dalam bahasa Indonesia.',
                });
                const text = await session.prompt(
                    'Tulis hanya satu paragraf bahasa Indonesia sekitar 65 sampai 85 kata. ' +
                    'Tanpa judul dan tanpa daftar. ' + prompt,
                    {maxTokens: 220, temperature: 0.5, seed: 1029 + index},
                );
                if (!text.trim()) throw new Error('Generator produced an empty response.');
                generated.push({
                    text: text.trim(), label: 1, group,
                    source, license: 'Research-generated; see generation source and model card.',
                });
            } finally {
                await context.dispose();
            }
        }
        fs.writeFileSync(out, generated.map(row => JSON.stringify(row)).join('\n') + '\n',
            {flag: 'wx', mode: 0o600});
        console.log('Generated', generated.length, 'AI-only examples from', source);
    } finally {
        await model.dispose();
    }
} finally {
    await llama.dispose();
}
