using System;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Shapes;

namespace CalculatorGui.View
{
    /// <summary>
    /// Interaction logic for GraphingView.xaml
    /// </summary>
    public partial class GraphingView : UserControl
    {
        private double _xMin = -10;
        private double _xMax = 10;
        private double _yMin = -10;
        private double _yMax = 10;
        private bool _isDraggingGraph = false;
        private Point _lastMousePos;

        public GraphingView()
        {
            InitializeComponent();
        }

        private void GraphCanvas_SizeChanged(object sender, SizeChangedEventArgs e)
        {
            RedrawGraph();
        }

        private void TxtGraphFunction_TextChanged(object sender, TextChangedEventArgs e)
        {
            RedrawGraph();
        }

        private void GraphCanvas_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            _isDraggingGraph = true;
            _lastMousePos = e.GetPosition(GraphCanvas);
            GraphCanvas.CaptureMouse();
            HideTracker();
        }

        private void GraphCanvas_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
        {
            _isDraggingGraph = false;
            GraphCanvas.ReleaseMouseCapture();
        }

        private void GraphCanvas_MouseMove(object sender, MouseEventArgs e)
        {
            Point currentPos = e.GetPosition(GraphCanvas);

            if (_isDraggingGraph)
            {
                HideTracker();

                double dx = currentPos.X - _lastMousePos.X;
                double dy = currentPos.Y - _lastMousePos.Y;

                double xRange = _xMax - _xMin;
                double yRange = _yMax - _yMin;

                double deltaX = (dx / GraphCanvas.ActualWidth) * xRange;
                double deltaY = (dy / GraphCanvas.ActualHeight) * yRange;

                _xMin -= deltaX;
                _xMax -= deltaX;
                _yMin += deltaY;
                _yMax += deltaY;

                _lastMousePos = currentPos;
                RedrawGraph();
            }
            else
            {
                UpdateTracker(currentPos);
            }
        }

        private void GraphCanvas_MouseWheel(object sender, MouseWheelEventArgs e)
        {
            double factor = e.Delta > 0 ? 0.85 : 1.1764705882352942;
            
            Point mousePos = e.GetPosition(GraphCanvas);
            double width = GraphCanvas.ActualWidth;
            double height = GraphCanvas.ActualHeight;
            
            if (width <= 0 || height <= 0) return;

            // Mathematical coordinates at current mouse position
            double mx = _xMin + (mousePos.X / width) * (_xMax - _xMin);
            double my = _yMin + (1.0 - (mousePos.Y / height)) * (_yMax - _yMin);

            double newXRange = (_xMax - _xMin) * factor;
            double newYRange = (_yMax - _yMin) * factor;

            _xMin = mx - (mousePos.X / width) * newXRange;
            _xMax = _xMin + newXRange;
            _yMin = my - (1.0 - (mousePos.Y / height)) * newYRange;
            _yMax = _yMin + newYRange;

            RedrawGraph();

            // Force update tracker after zoom if mouse is still on canvas
            UpdateTracker(mousePos);
        }

        private void GraphCanvas_MouseLeave(object sender, MouseEventArgs e)
        {
            HideTracker();
        }

        private void UpdateTracker(Point pos)
        {
            if (GraphCanvas == null || TrackingCanvas == null) return;

            double width = GraphCanvas.ActualWidth;
            double height = GraphCanvas.ActualHeight;

            if (width <= 0 || height <= 0)
            {
                HideTracker();
                return;
            }

            // Calculate mathematical coordinates
            double xVal = _xMin + (pos.X / width) * (_xMax - _xMin);
            double yVal = _yMin + (1.0 - (pos.Y / height)) * (_yMax - _yMin);

            // Format coordinates
            TxtTrackerCoords.Text = $"x: {xVal.ToString("G5", System.Globalization.CultureInfo.InvariantCulture)}\ny: {yVal.ToString("G5", System.Globalization.CultureInfo.InvariantCulture)}";

            // Position lines
            TrackerLineY.X1 = pos.X;
            TrackerLineY.X2 = pos.X;
            TrackerLineY.Y1 = 0;
            TrackerLineY.Y2 = height;

            TrackerLineX.X1 = 0;
            TrackerLineX.X2 = width;
            TrackerLineX.Y1 = pos.Y;
            TrackerLineX.Y2 = pos.Y;

            // Position tooltip, avoiding clipping at edges
            TrackerTooltip.UpdateLayout(); // Ensure ActualWidth and ActualHeight are calculated
            double tooltipLeft = pos.X + 15;
            double tooltipTop = pos.Y + 15;

            if (tooltipLeft + TrackerTooltip.ActualWidth > width)
            {
                tooltipLeft = pos.X - TrackerTooltip.ActualWidth - 15;
            }
            if (tooltipTop + TrackerTooltip.ActualHeight > height)
            {
                tooltipTop = pos.Y - TrackerTooltip.ActualHeight - 15;
            }

            // Clamping tooltip to canvas bounds
            if (tooltipLeft < 0) tooltipLeft = 5;
            if (tooltipTop < 0) tooltipTop = 5;

            Canvas.SetLeft(TrackerTooltip, tooltipLeft);
            Canvas.SetTop(TrackerTooltip, tooltipTop);

            // Show tracker elements
            TrackerLineX.Visibility = Visibility.Visible;
            TrackerLineY.Visibility = Visibility.Visible;
            TrackerTooltip.Visibility = Visibility.Visible;
        }

        private void HideTracker()
        {
            if (TrackerLineX != null) TrackerLineX.Visibility = Visibility.Collapsed;
            if (TrackerLineY != null) TrackerLineY.Visibility = Visibility.Collapsed;
            if (TrackerTooltip != null) TrackerTooltip.Visibility = Visibility.Collapsed;
        }

        private void Zoom(double factor)
        {
            double xCenter = (_xMin + _xMax) / 2.0;
            double yCenter = (_yMin + _yMax) / 2.0;

            double halfXRange = (_xMax - _xMin) * factor / 2.0;
            double halfYRange = (_yMax - _yMin) * factor / 2.0;

            _xMin = xCenter - halfXRange;
            _xMax = xCenter + halfXRange;
            _yMin = yCenter - halfYRange;
            _yMax = yCenter + halfYRange;

            RedrawGraph();
        }

        private void BtnZoomIn_Click(object sender, RoutedEventArgs e)
        {
            Zoom(0.75);
        }

        private void BtnZoomOut_Click(object sender, RoutedEventArgs e)
        {
            Zoom(1.3333333333333333);
        }

        private void BtnZoomReset_Click(object sender, RoutedEventArgs e)
        {
            _xMin = -10;
            _xMax = 10;
            _yMin = -10;
            _yMax = 10;
            RedrawGraph();
        }

        private void RedrawGraph()
        {
            if (GraphCanvas == null || !GraphCanvas.IsVisible) return;

            double width = GraphCanvas.ActualWidth;
            double height = GraphCanvas.ActualHeight;

            if (width <= 0 || height <= 0) return;

            GraphCanvas.Children.Clear();

            // 1. Draw adaptive grid and labels
            double step = CalculatorBridge.CalculateGridStep(_xMin, _xMax);
            double startX = Math.Ceiling(_xMin / step) * step;
            double endX = Math.Floor(_xMax / step) * step;

            for (double xVal = startX; xVal <= endX; xVal += step)
            {
                double px = (xVal - _xMin) / (_xMax - _xMin) * width;

                var line = new Line
                {
                    X1 = px,
                    Y1 = 0,
                    X2 = px,
                    Y2 = height,
                    Stroke = (Math.Abs(xVal) < 1e-9) ? (Brush)FindResource("SciButtonTextBrush") : (Brush)FindResource("BorderBrush"),
                    StrokeThickness = (Math.Abs(xVal) < 1e-9) ? 2 : 0.5,
                    Opacity = (Math.Abs(xVal) < 1e-9) ? 0.8 : 0.4
                };
                GraphCanvas.Children.Add(line);

                if (Math.Abs(xVal) > 1e-9)
                {
                    var text = new TextBlock
                    {
                        Text = xVal.ToString("G5", System.Globalization.CultureInfo.InvariantCulture),
                        Foreground = (Brush)FindResource("TextSecBrush"),
                        FontSize = 10
                    };
                    double pyAxis = (0.0 - _yMin) / (_yMax - _yMin);
                    double py = (1.0 - pyAxis) * height;
                    if (py < 10) py = 10;
                    if (py > height - 20) py = height - 20;

                    Canvas.SetLeft(text, px + 5);
                    Canvas.SetTop(text, py + 2);
                    GraphCanvas.Children.Add(text);
                }
            }

            double yStep = CalculatorBridge.CalculateGridStep(_yMin, _yMax);
            double startY = Math.Ceiling(_yMin / yStep) * yStep;
            double endY = Math.Floor(_yMax / yStep) * yStep;

            for (double yVal = startY; yVal <= endY; yVal += yStep)
            {
                double pyAxis = (yVal - _yMin) / (_yMax - _yMin);
                double py = (1.0 - pyAxis) * height;

                var line = new Line
                {
                    X1 = 0,
                    Y1 = py,
                    X2 = width,
                    Y2 = py,
                    Stroke = (Math.Abs(yVal) < 1e-9) ? (Brush)FindResource("SciButtonTextBrush") : (Brush)FindResource("BorderBrush"),
                    StrokeThickness = (Math.Abs(yVal) < 1e-9) ? 2 : 0.5,
                    Opacity = (Math.Abs(yVal) < 1e-9) ? 0.8 : 0.4
                };
                GraphCanvas.Children.Add(line);

                if (Math.Abs(yVal) > 1e-9)
                {
                    var text = new TextBlock
                    {
                        Text = yVal.ToString("G5", System.Globalization.CultureInfo.InvariantCulture),
                        Foreground = (Brush)FindResource("TextSecBrush"),
                        FontSize = 10
                    };
                    double pxAxis = (0.0 - _xMin) / (_xMax - _xMin);
                    double px = pxAxis * width;
                    if (px < 10) px = 10;
                    if (px > width - 40) px = width - 40;

                    Canvas.SetLeft(text, px + 5);
                    Canvas.SetTop(text, py - 12);
                    GraphCanvas.Children.Add(text);
                }
            }

            // Draw intersection '0' label
            double px0 = (0.0 - _xMin) / (_xMax - _xMin) * width;
            double py0 = (1.0 - (0.0 - _yMin) / (_yMax - _yMin)) * height;
            if (px0 >= 0 && px0 <= width && py0 >= 0 && py0 <= height)
            {
                var text0 = new TextBlock
                {
                    Text = "0",
                    Foreground = (Brush)FindResource("TextSecBrush"),
                    FontSize = 10
                };
                Canvas.SetLeft(text0, px0 - 12);
                Canvas.SetTop(text0, py0 + 2);
                GraphCanvas.Children.Add(text0);
            }

            // 2. Draw function curve using FFI
            string functionExpr = TxtGraphFunction.Text.Trim();
            if (!string.IsNullOrEmpty(functionExpr))
            {
                int steps = 500;
                Polyline? currentPolyline = null;

                // Detect mode
                bool isParametric = false;
                string xExpr = "", yExpr = "";
                string[] parts = functionExpr.Split(new char[] { ';', ',' }, StringSplitOptions.RemoveEmptyEntries);
                if (parts.Length == 2 && (parts[0].Contains("t") || parts[1].Contains("t")))
                {
                    isParametric = true;
                    xExpr = parts[0].Trim();
                    yExpr = parts[1].Trim();
                }

                bool isPolar = functionExpr.Contains("theta") || functionExpr.Contains("θ");
                bool isVertical = !isParametric && !isPolar && functionExpr.Contains("y");

                if (isParametric)
                {
                    double tMin = -10.0;
                    double tMax = 10.0;
                    if (functionExpr.Contains("sin") || functionExpr.Contains("cos") || functionExpr.Contains("tan") || functionExpr.Contains("pi") || functionExpr.Contains("π"))
                    {
                        tMin = 0.0;
                        tMax = 2.0 * Math.PI;
                    }

                    for (int i = 0; i <= steps; i++)
                    {
                        double tVal = tMin + ((double)i / steps) * (tMax - tMin);
                        double xVal = CalculatorBridge.EvaluateParametric(xExpr, tVal);
                        double yVal = CalculatorBridge.EvaluateParametric(yExpr, tVal);

                        if (!double.IsNaN(xVal) && !double.IsInfinity(xVal) && !double.IsNaN(yVal) && !double.IsInfinity(yVal))
                        {
                            double px = (xVal - _xMin) / (_xMax - _xMin) * width;
                            double py = (1.0 - (yVal - _yMin) / (_yMax - _yMin)) * height;

                            if (px >= -width * 2 && px <= width * 3 && py >= -height * 2 && py <= height * 3)
                            {
                                if (currentPolyline == null)
                                {
                                    currentPolyline = new Polyline
                                    {
                                        Stroke = (Brush)FindResource("AccentBrush"),
                                        StrokeThickness = 2.5
                                    };
                                    GraphCanvas.Children.Add(currentPolyline);
                                }
                                currentPolyline.Points.Add(new Point(px, py));
                            }
                            else
                            {
                                currentPolyline = null;
                            }
                        }
                        else
                        {
                            currentPolyline = null;
                        }
                    }
                }
                else if (isPolar)
                {
                    double thetaMin = 0.0;
                    double thetaMax = 2.0 * Math.PI;
                    if (functionExpr.Contains("spiral") || functionExpr.Contains("theta *") || functionExpr.Contains("θ *"))
                    {
                        thetaMax = 4.0 * Math.PI;
                    }

                    for (int i = 0; i <= steps; i++)
                    {
                        double thetaVal = thetaMin + ((double)i / steps) * (thetaMax - thetaMin);
                        double r = CalculatorBridge.EvaluatePolar(functionExpr, thetaVal);

                        if (!double.IsNaN(r) && !double.IsInfinity(r))
                        {
                            double xVal = r * Math.Cos(thetaVal);
                            double yVal = r * Math.Sin(thetaVal);

                            double px = (xVal - _xMin) / (_xMax - _xMin) * width;
                            double py = (1.0 - (yVal - _yMin) / (_yMax - _yMin)) * height;

                            if (px >= -width * 2 && px <= width * 3 && py >= -height * 2 && py <= height * 3)
                            {
                                if (currentPolyline == null)
                                {
                                    currentPolyline = new Polyline
                                    {
                                        Stroke = (Brush)FindResource("AccentBrush"),
                                        StrokeThickness = 2.5
                                    };
                                    GraphCanvas.Children.Add(currentPolyline);
                                }
                                currentPolyline.Points.Add(new Point(px, py));
                            }
                            else
                            {
                                currentPolyline = null;
                            }
                        }
                        else
                        {
                            currentPolyline = null;
                        }
                    }
                }
                else if (isVertical)
                {
                    for (int i = 0; i <= steps; i++)
                    {
                        double t = (double)i / steps;
                        double yVal = _yMin + t * (_yMax - _yMin);
                        double xVal = CalculatorBridge.EvaluateWithVar(functionExpr, yVal);

                        if (!double.IsNaN(xVal) && !double.IsInfinity(xVal))
                        {
                            double px = (xVal - _xMin) / (_xMax - _xMin) * width;
                            double py = (1.0 - t) * height;

                            if (px >= -width * 2 && px <= width * 3)
                            {
                                if (currentPolyline == null)
                                {
                                    currentPolyline = new Polyline
                                    {
                                        Stroke = (Brush)FindResource("AccentBrush"),
                                        StrokeThickness = 2.5
                                    };
                                    GraphCanvas.Children.Add(currentPolyline);
                                }
                                currentPolyline.Points.Add(new Point(px, py));
                            }
                            else
                            {
                                currentPolyline = null;
                            }
                        }
                        else
                        {
                            currentPolyline = null;
                        }
                    }
                }
                else
                {
                    for (int i = 0; i <= steps; i++)
                    {
                        double t = (double)i / steps;
                        double xVal = _xMin + t * (_xMax - _xMin);
                        double yVal = CalculatorBridge.EvaluateWithVar(functionExpr, xVal);

                        if (!double.IsNaN(yVal) && !double.IsInfinity(yVal))
                        {
                            double px = t * width;
                            double py = (1.0 - (yVal - _yMin) / (_yMax - _yMin)) * height;

                            if (py >= -height * 2 && py <= height * 3)
                            {
                                if (currentPolyline == null)
                                {
                                    currentPolyline = new Polyline
                                    {
                                        Stroke = (Brush)FindResource("AccentBrush"),
                                        StrokeThickness = 2.5
                                    };
                                    GraphCanvas.Children.Add(currentPolyline);
                                }
                                currentPolyline.Points.Add(new Point(px, py));
                            }
                            else
                            {
                                currentPolyline = null;
                            }
                        }
                        else
                        {
                            currentPolyline = null;
                        }
                    }
                }
            }
        }
    }
}
