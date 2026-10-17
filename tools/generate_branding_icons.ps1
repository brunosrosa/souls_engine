# tools/generate_branding_icons.ps1
# Gerador Automatizado de Assets e Ícones de Branding para o Souls Engine v7
# Produz variações de cores dos olhos (Ciano, Verde, Âmbar, Vermelho, Roxo)
# e aplica recorte com BORDAS ARREDONDADAS (Squircle) e FUNDO TRANSPARENTE nos 4 cantos externos.
# Gera PNGs em múltiplos tamanhos + arquivos .ICO multi-resolução para Windows Systray / Launcher.

$srcCandidates = @(
    "C:\Users\rosas\.gemini\antigravity-ide\brain\9879e3dc-0528-4554-8c33-7bb3608b04bc\.user_uploaded\media_1792209760265.jpg",
    "C:\Users\rosas\.gemini\antigravity-ide\brain\4e3e1bda-872a-422e-ac39-cf8c34e69fa9\.user_uploaded\media_1791834488261.jpg"
)

$srcImage = $srcCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $srcImage) {
    Write-Error "Imagem fonte do mascote não encontrada!"
    exit 1
}

$outDir = "Z:\souls_engine\assets\branding"

if (-not (Test-Path $outDir)) {
    New-Item -ItemType Directory -Path $outDir -Force | Out-Null
}

$csharpCode = @'
using System;
using System.IO;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Collections.Generic;

public class SoulsIconBuilder {
    // Converte RGB para HSL, altera o Hue nos pixels com saturação (olhos) e converte de volta para RGB
    public static Bitmap ShiftHue(Bitmap src, float targetHue) {
        Bitmap dest = new Bitmap(src.Width, src.Height, PixelFormat.Format32bppArgb);
        BitmapData srcData = src.LockBits(new Rectangle(0, 0, src.Width, src.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
        BitmapData destData = dest.LockBits(new Rectangle(0, 0, dest.Width, dest.Height), ImageLockMode.WriteOnly, PixelFormat.Format32bppArgb);

        int bytes = Math.Abs(srcData.Stride) * src.Height;
        byte[] srcRgb = new byte[bytes];
        byte[] destRgb = new byte[bytes];

        System.Runtime.InteropServices.Marshal.Copy(srcData.Scan0, srcRgb, 0, bytes);

        for (int i = 0; i < bytes; i += 4) {
            byte b = srcRgb[i];
            byte g = srcRgb[i + 1];
            byte r = srcRgb[i + 2];
            byte a = srcRgb[i + 3];

            float rf = r / 255f;
            float gf = g / 255f;
            float bf = b / 255f;

            float max = Math.Max(rf, Math.Max(gf, bf));
            float min = Math.Min(rf, Math.Min(gf, bf));
            float delta = max - min;
            float sat = max == 0 ? 0 : delta / max;
            float lum = (max + min) / 2f;

            // Se for pixel dos olhos (saturado e luminoso)
            if (sat > 0.14f && max > 0.12f) {
                float c = (1f - Math.Abs(2f * lum - 1f)) * sat;
                float x = c * (1f - Math.Abs((targetHue / 60f) % 2f - 1f));
                float m = lum - c / 2f;

                float r1 = 0, g1 = 0, b1 = 0;
                if (targetHue >= 0 && targetHue < 60) { r1 = c; g1 = x; b1 = 0; }
                else if (targetHue >= 60 && targetHue < 120) { r1 = x; g1 = c; b1 = 0; }
                else if (targetHue >= 120 && targetHue < 180) { r1 = 0; g1 = c; b1 = x; }
                else if (targetHue >= 180 && targetHue < 240) { r1 = 0; g1 = x; b1 = c; }
                else if (targetHue >= 240 && targetHue < 300) { r1 = x; g1 = 0; b1 = c; }
                else { r1 = c; g1 = 0; b1 = x; }

                destRgb[i] = (byte)Math.Min(255, Math.Max(0, (b1 + m) * 255));
                destRgb[i + 1] = (byte)Math.Min(255, Math.Max(0, (g1 + m) * 255));
                destRgb[i + 2] = (byte)Math.Min(255, Math.Max(0, (r1 + m) * 255));
                destRgb[i + 3] = a;
            } else {
                // Preserva o corpo preto/cinza fosco idêntico ao original
                destRgb[i] = b;
                destRgb[i + 1] = g;
                destRgb[i + 2] = r;
                destRgb[i + 3] = a;
            }
        }

        System.Runtime.InteropServices.Marshal.Copy(destRgb, 0, destData.Scan0, bytes);
        src.UnlockBits(srcData);
        dest.UnlockBits(destData);
        return dest;
    }

    // Cria GraphicsPath de retângulo com cantos arredondados (squircle)
    public static GraphicsPath CreateRoundedPath(RectangleF rect, float radius) {
        GraphicsPath path = new GraphicsPath();
        float d = radius * 2f;
        path.AddArc(rect.X, rect.Y, d, d, 180, 90);
        path.AddArc(rect.Right - d, rect.Y, d, d, 270, 90);
        path.AddArc(rect.Right - d, rect.Bottom - d, d, d, 0, 90);
        path.AddArc(rect.X, rect.Bottom - d, d, d, 90, 90);
        path.CloseFigure();
        return path;
    }

    // Aplica o recorte de bordas arredondadas e torna o exterior 100% transparente
    public static Bitmap ApplyRoundedCorners(Bitmap src, float x, float y, float w, float h, float radius) {
        Bitmap dest = new Bitmap(src.Width, src.Height, PixelFormat.Format32bppArgb);
        using (Graphics g = Graphics.FromImage(dest)) {
            g.Clear(Color.Transparent);
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.InterpolationMode = InterpolationMode.HighQualityBicubic;
            g.PixelOffsetMode = PixelOffsetMode.HighQuality;
            g.CompositingQuality = CompositingQuality.HighQuality;

            RectangleF rect = new RectangleF(x, y, w, h);
            using (GraphicsPath path = CreateRoundedPath(rect, radius)) {
                using (TextureBrush tb = new TextureBrush(src)) {
                    g.FillPath(tb, path);
                }
            }
        }
        return dest;
    }

    public static Bitmap ResizeImage(Bitmap src, int width, int height) {
        Bitmap dest = new Bitmap(width, height, PixelFormat.Format32bppArgb);
        using (Graphics g = Graphics.FromImage(dest)) {
            g.Clear(Color.Transparent);
            g.CompositingQuality = CompositingQuality.HighQuality;
            g.InterpolationMode = InterpolationMode.HighQualityBicubic;
            g.SmoothingMode = SmoothingMode.HighQuality;
            g.PixelOffsetMode = PixelOffsetMode.HighQuality;
            g.DrawImage(src, 0, 0, width, height);
        }
        return dest;
    }

    // Cria um arquivo .ICO válido do Windows contendo múltiplos tamanhos embutidos
    public static void SaveAsIcon(Bitmap masterBmp, string icoPath, int[] sizes) {
        using (FileStream fs = new FileStream(icoPath, FileMode.Create))
        using (BinaryWriter bw = new BinaryWriter(fs)) {
            // ICO Header: 0 (reserved), 1 (icon type), count
            bw.Write((short)0);
            bw.Write((short)1);
            bw.Write((short)sizes.Length);

            List<byte[]> pngBuffers = new List<byte[]>();
            foreach (int size in sizes) {
                using (Bitmap resized = ResizeImage(masterBmp, size, size))
                using (MemoryStream ms = new MemoryStream()) {
                    resized.Save(ms, ImageFormat.Png);
                    pngBuffers.Add(ms.ToArray());
                }
            }

            int offset = 6 + (16 * sizes.Length);
            for (int i = 0; i < sizes.Length; i++) {
                int s = sizes[i];
                byte w = (byte)(s >= 256 ? 0 : s);
                byte h = (byte)(s >= 256 ? 0 : s);
                bw.Write(w);           // Width
                bw.Write(h);           // Height
                bw.Write((byte)0);     // Colors
                bw.Write((byte)0);     // Reserved
                bw.Write((short)1);    // Color planes
                bw.Write((short)32);   // Bits per pixel
                bw.Write(pngBuffers[i].Length); // Size of image data
                bw.Write(offset);      // Offset of image data
                offset += pngBuffers[i].Length;
            }

            for (int i = 0; i < pngBuffers.Count; i++) {
                bw.Write(pngBuffers[i]);
            }
        }
    }
}
'@

$runnerScript = @"
using namespace System.Drawing
using namespace System.Drawing.Imaging

[System.Reflection.Assembly]::LoadWithPartialName('System.Drawing') | Out-Null
Add-Type -TypeDefinition @'
$csharpCode
'@ -ReferencedAssemblies System.Drawing

`$src = '$srcImage'
`$out = '$outDir'

Write-Host "Carregando imagem fonte: `$src" -ForegroundColor Cyan
`$orig = [Bitmap]::new(`$src)

# Cores: Nome, Hue, Descrição
`$variants = @(
    @{ Name = 'cyan';   Hue = -1;  Desc = 'Default / Idle / Standby' },
    @{ Name = 'green';  Hue = 135; Desc = 'Online / Healthy / Executing' },
    @{ Name = 'amber';  Hue = 38;  Desc = 'Subagent Active / Busy / Routing' },
    @{ Name = 'red';    Hue = 0;   Desc = 'Thermal Alert >= 78C / Error' },
    @{ Name = 'purple'; Hue = 285; Desc = 'Deep Socratic Thinking / Memory' }
)

`$sizes = @(16, 24, 32, 48, 64, 128, 256, 512)
`$icoSizes = [int[]]@(16, 24, 32, 48, 64, 128, 256)

foreach (`$v in `$variants) {
    Write-Host "Processando variante: `$(`$v.Name) (`$(`$v.Desc))..."
    `$rawColor = if (`$v.Hue -lt 0) { `$orig } else { [SoulsIconBuilder]::ShiftHue(`$orig, [float]`$v.Hue) }

    # Aplica o recorte de bordas arredondadas (Squircle) com fundo transparente nos 4 cantos
    `$colorBmp = [SoulsIconBuilder]::ApplyRoundedCorners(`$rawColor, 60.0, 50.0, 878.0, 880.0, 180.0)

    # Salva o master PNG de 1024x1024
    `$masterPath = Join-Path `$out ("souls_icon_`$(`$v.Name)_1024.png")
    `$colorBmp.Save(`$masterPath, [ImageFormat]::Png)

    # Também salva no nome padrão para interoperabilidade
    `$stdPath = Join-Path `$out ("souls_icon_`$(`$v.Name).png")
    `$colorBmp.Save(`$stdPath, [ImageFormat]::Png)

    # Salva os PNGs nas resoluções úteis
    foreach (`$s in `$sizes) {
        `$resized = [SoulsIconBuilder]::ResizeImage(`$colorBmp, `$s, `$s)
        `$pngPath = Join-Path `$out ("souls_icon_`$(`$v.Name)_`$s.png")
        `$resized.Save(`$pngPath, [ImageFormat]::Png)
        `$resized.Dispose()
    }

    # Salva o arquivo Windows .ICO multi-resolução
    `$icoPath = Join-Path `$out ("souls_icon_`$(`$v.Name).ico")
    [SoulsIconBuilder]::SaveAsIcon(`$colorBmp, `$icoPath, `$icoSizes)

    `$colorBmp.Dispose()
    if (`$v.Hue -ge 0) {
        `$rawColor.Dispose()
    }
}

`$orig.Dispose()
Write-Host "Processamento concluído com sucesso! Ícones com bordas arredondadas e cantos transparentes gerados." -ForegroundColor Green
"@

$runnerScriptPath = "Z:\souls_engine\tools\_run_branding.ps1"
Set-Content -Path $runnerScriptPath -Value $runnerScript -Encoding UTF8
powershell.exe -ExecutionPolicy Bypass -File $runnerScriptPath
Remove-Item -Path $runnerScriptPath -Force -ErrorAction SilentlyContinue
