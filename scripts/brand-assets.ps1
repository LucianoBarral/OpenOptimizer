# Gera os assets de marca a partir de docs/design/logo-source.png (logo branca sobre preto).
# Uso: powershell -ExecutionPolicy Bypass -File scripts\brand-assets.ps1
# Saídas em docs/design/brand/:
#   logo-mark.png   logo com alpha (fundo transparente), 1024x1024
#   logo-dial.png   idem, SEM o ponteiro (o ponteiro é redesenhado em SVG e animado na UI)
#   needle.json     geometria do ponteiro no espaço 1024x1024
#   icon-1024.png   ícone do app (logo sobre quadrado arredondado #0B0C0E)
#   nsis-sidebar.bmp (164x314) e nsis-header.bmp (150x57) para o instalador
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$src  = Join-Path $root 'docs\design\logo-source.png'
$out  = Join-Path $root 'docs\design\brand'
New-Item -ItemType Directory -Force $out | Out-Null

Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;

public static class Brand {
    static byte[] Read(Bitmap b, out int stride) {
        var d = b.LockBits(new Rectangle(0,0,b.Width,b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
        stride = d.Stride; var buf = new byte[stride*b.Height];
        Marshal.Copy(d.Scan0, buf, 0, buf.Length); b.UnlockBits(d); return buf;
    }
    static Bitmap Write(byte[] buf, int w, int h, int stride) {
        var b = new Bitmap(w, h, PixelFormat.Format32bppArgb);
        var d = b.LockBits(new Rectangle(0,0,w,h), ImageLockMode.WriteOnly, PixelFormat.Format32bppArgb);
        Marshal.Copy(buf, 0, d.Scan0, buf.Length); b.UnlockBits(d); return b;
    }
    static int Lum(byte[] p, int i) { return Math.Max(p[i], Math.Max(p[i+1], p[i+2])); }

    // Acha o ponteiro: raio claro mais longo a partir do hub, entre 10° e 80° acima da horizontal (lado direito).
    public static double[] FindNeedle(Bitmap b, int hx, int hy, int hubR) {
        int stride; var p = Read(b, out stride);
        double bestA = 0; int bestR = 0;
        for (double deg = 10; deg <= 80; deg += 0.25) {
            double a = -deg * Math.PI / 180; int r = hubR;
            for (; r < 400; r++) {
                int x = (int)Math.Round(hx + r*Math.Cos(a)), y = (int)Math.Round(hy + r*Math.Sin(a));
                if (x<0||y<0||x>=b.Width||y>=b.Height) break;
                if (Lum(p, y*stride + x*4) < 110) break;
            }
            if (r > bestR) { bestR = r; bestA = a; }
        }
        // meia-largura perto da base (r = hubR + 12), medida na perpendicular
        double px = -Math.Sin(bestA), py = Math.Cos(bestA);
        double bx = hx + (hubR+12)*Math.Cos(bestA), by = hy + (hubR+12)*Math.Sin(bestA);
        int hw = 0;
        for (; hw < 40; hw++) {
            int x = (int)Math.Round(bx + hw*px), y = (int)Math.Round(by + hw*py);
            if (Lum(p, y*stride + x*4) < 110) break;
        }
        return new double[] { hx + bestR*Math.Cos(bestA), hy + bestR*Math.Sin(bestA), hw };
    }

    // Pinta de preto o ponteiro (segmento afunilado) e o hub, com margem.
    public static Bitmap EraseNeedle(Bitmap b, int hx, int hy, int hubR, double tx, double ty, double hw0, int margin) {
        int stride; var p = Read(b, out stride);
        double vx = tx-hx, vy = ty-hy, len2 = vx*vx+vy*vy;
        for (int y = 0; y < b.Height; y++) for (int x = 0; x < b.Width; x++) {
            double dx = x-hx, dy = y-hy;
            bool hit = dx*dx+dy*dy <= (hubR+margin)*(hubR+margin);
            if (!hit) {
                double t = (dx*vx+dy*vy)/len2;
                if (t >= 0 && t <= 1.02) {
                    double cx = hx+t*vx, cy = hy+t*vy, d = Math.Sqrt((x-cx)*(x-cx)+(y-cy)*(y-cy));
                    double half = hw0 + (2 - hw0)*Math.Min(t,1) + margin;
                    hit = d <= half;
                }
            }
            if (hit) { int i = y*stride + x*4; p[i]=0; p[i+1]=0; p[i+2]=0; p[i+3]=255; }
        }
        return Write(p, b.Width, b.Height, stride);
    }

    // Luminância → alpha (branco sobre transparente), recortando um quadrado e escalando para "size".
    public static Bitmap ToAlpha(Bitmap b, int ox, int oy, int side, int size) {
        var crop = new Bitmap(size, size, PixelFormat.Format32bppArgb);
        using (var g = Graphics.FromImage(crop)) {
            g.InterpolationMode = System.Drawing.Drawing2D.InterpolationMode.HighQualityBicubic;
            g.Clear(Color.Black);
            g.DrawImage(b, new Rectangle(0,0,size,size), new Rectangle(ox,oy,side,side), GraphicsUnit.Pixel);
        }
        int stride; var p = Read(crop, out stride);
        for (int i = 0; i < p.Length; i += 4) {
            int l = Math.Max(p[i], Math.Max(p[i+1], p[i+2]));
            int a = l < 14 ? 0 : Math.Min(255, (l - 14) * 255 / 230);   // corta o ruído do fundo preto
            // mantém o tom metálico: cor = cinza da luminância normalizada pelo alpha
            byte c = (byte)(a == 0 ? 0 : Math.Min(255, l * 255 / Math.Max(1, (a * 241 / 255) + 14)));
            p[i] = c; p[i+1] = c; p[i+2] = c; p[i+3] = (byte)a;
        }
        crop.Dispose();
        return Write(p, size, size, stride);
    }

    public static Bitmap Icon(Bitmap mark, int size, double logoScale) {
        var b = new Bitmap(size, size, PixelFormat.Format32bppArgb);
        using (var g = Graphics.FromImage(b)) {
            g.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.AntiAlias;
            g.InterpolationMode = System.Drawing.Drawing2D.InterpolationMode.HighQualityBicubic;
            g.Clear(Color.Transparent);
            int r = (int)(size * 0.225);
            using (var path = new System.Drawing.Drawing2D.GraphicsPath()) {
                path.AddArc(0, 0, 2*r, 2*r, 180, 90); path.AddArc(size-2*r, 0, 2*r, 2*r, 270, 90);
                path.AddArc(size-2*r, size-2*r, 2*r, 2*r, 0, 90); path.AddArc(0, size-2*r, 2*r, 2*r, 90, 90);
                path.CloseFigure();
                using (var br = new SolidBrush(Color.FromArgb(255, 11, 12, 14))) g.FillPath(br, path);
            }
            int s = (int)(size * logoScale), o = (size - s) / 2;
            g.DrawImage(mark, o, o, s, s);
        }
        return b;
    }

    public static Bitmap Panel(Bitmap mark, int w, int h, int logo, int lx, int ly, string text, float fontPx, int tx, int ty) {
        var b = new Bitmap(w, h, PixelFormat.Format24bppRgb);
        using (var g = Graphics.FromImage(b)) {
            g.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.AntiAlias;
            g.InterpolationMode = System.Drawing.Drawing2D.InterpolationMode.HighQualityBicubic;
            g.TextRenderingHint = System.Drawing.Text.TextRenderingHint.AntiAliasGridFit;
            g.Clear(Color.FromArgb(11, 12, 14));
            g.DrawImage(mark, lx, ly, logo, logo);
            if (!string.IsNullOrEmpty(text)) using (var f = new Font("Segoe UI Semibold", fontPx, GraphicsUnit.Pixel))
                using (var br = new SolidBrush(Color.FromArgb(238, 243, 247))) g.DrawString(text, f, br, tx, ty);
        }
        return b;
    }
}
'@

$b = [System.Drawing.Bitmap]::FromFile($src)
# Hub medido na imagem original (1254x1254): centro (618,660), raio ~38.
$hx = 618; $hy = 660; $hubR = 38
$n = [Brand]::FindNeedle($b, $hx, $hy, $hubR)
"ponteiro: ponta ({0:N0},{1:N0}), meia-largura na base {2}" -f $n[0], $n[1], $n[2]
$dial = [Brand]::EraseNeedle($b, $hx, $hy, $hubR, $n[0], $n[1], $n[2], 7)

# Quadrado de recorte centrado no bbox da logo (x 264..1044, y 332..932) com folga.
$side = 880; $ox = 654 - $side/2; $oy = 632 - $side/2; $size = 1024
$mark = [Brand]::ToAlpha($b, $ox, $oy, $side, $size)
$dialA = [Brand]::ToAlpha($dial, $ox, $oy, $side, $size)
$mark.Save((Join-Path $out 'logo-mark.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$dialA.Save((Join-Path $out 'logo-dial.png'), [System.Drawing.Imaging.ImageFormat]::Png)

$k = $size / $side
$needle = [ordered]@{
  hubX = [math]::Round(($hx - $ox) * $k, 1); hubY = [math]::Round(($hy - $oy) * $k, 1); hubR = [math]::Round($hubR * $k, 1)
  tipX = [math]::Round(($n[0] - $ox) * $k, 1); tipY = [math]::Round(($n[1] - $oy) * $k, 1)
  baseHalfWidth = [math]::Round($n[2] * $k, 1)
}
$needle | ConvertTo-Json | Set-Content (Join-Path $out 'needle.json') -Encoding utf8
"needle.json: " + ($needle | ConvertTo-Json -Compress)

$icon = [Brand]::Icon($mark, 1024, 0.86)
$icon.Save((Join-Path $out 'icon-1024.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$side164 = [Brand]::Panel($mark, 164, 314, 120, 22, 70, 'Open Optimizer', 15, 24, 200)
$side164.Save((Join-Path $out 'nsis-sidebar.bmp'), [System.Drawing.Imaging.ImageFormat]::Bmp)
$hdr = [Brand]::Panel($mark, 150, 57, 52, 94, 2, $null, 0, 0, 0)
$hdr.Save((Join-Path $out 'nsis-header.bmp'), [System.Drawing.Imaging.ImageFormat]::Bmp)
$b.Dispose(); $dial.Dispose(); $mark.Dispose(); $dialA.Dispose(); $icon.Dispose(); $side164.Dispose(); $hdr.Dispose()
Get-ChildItem $out | ForEach-Object { "{0,-18} {1,8:N0} bytes" -f $_.Name, $_.Length }
