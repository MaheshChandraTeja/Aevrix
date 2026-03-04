using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Windows.Input;
using Hummingbird.App.Models;
using Hummingbird.App.Services;

namespace Hummingbird.App.ViewModels;

public class ShellViewModel : INotifyPropertyChanged
{
    private readonly INavigationService _nav;
    private readonly ISettingsService _settings;

    public ObservableCollection<TabViewModel> Tabs { get; } = new();
    private TabViewModel? _selectedTab;
    public TabViewModel? SelectedTab
    {
        get => _selectedTab;
        set { _selectedTab = value; OnPropertyChanged(); OnPropertyChanged(nameof(SelectedTitle)); }
    }

    public string? SelectedTitle => SelectedTab?.Title;

    public ICommand NewTabCommand { get; }
    public ICommand CloseSelectedTabCommand { get; }
    public ICommand RefreshCommand { get; }
    public ICommand NavigateCommand { get; }

    public string AddressBarText { get; set; } = string.Empty;

    public ShellViewModel(INavigationService nav, ISettingsService settings)
    {
        _nav = nav;
        _settings = settings;

        NewTabCommand = new ActionCommand(_ => NewTab());
        CloseSelectedTabCommand = new ActionCommand(_ => CloseSelectedTab(), _ => SelectedTab != null);
        RefreshCommand = new ActionCommand(_ => SelectedTab?.RequestRender());
        NavigateCommand = new ActionCommand(async _ =>
        {
            if (SelectedTab == null) return;
            var (title, html) = _nav.Resolve(AddressBarText);
            SelectedTab.Model.Title = title;
            SelectedTab.Model.Html = html;
            OnPropertyChanged(nameof(SelectedTitle));
            await SelectedTab.RenderAsyncCurrentViewport();
        });

        
        var model = TabModel.CreateInitial();
        var vm = new TabViewModel(model, _settings);
        Tabs.Add(vm);
        SelectedTab = vm;
    }

    public void NewTab()
    {
        var m = new TabModel { Title = "New Tab", Html = "<div style='font-size:18px'>Blank</div>" };
        var vm = new TabViewModel(m, _settings);
        Tabs.Add(vm);
        SelectedTab = vm;
    }

    public void CloseSelectedTab()
    {
        if (SelectedTab == null) return;
        var idx = Tabs.IndexOf(SelectedTab);
        Tabs.Remove(SelectedTab);
        if (Tabs.Count == 0) return;
        SelectedTab = Tabs[Math.Clamp(idx - 1, 0, Tabs.Count - 1)];
    }

    public event PropertyChangedEventHandler? PropertyChanged;
    private void OnPropertyChanged([CallerMemberName] string? n = null)
        => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(n));

    private sealed class ActionCommand : ICommand
    {
        private readonly Action<object?> _exec;
        private readonly Func<object?, bool>? _can;
        public ActionCommand(Action<object?> exec, Func<object?, bool>? can = null) { _exec = exec; _can = can; }
        public bool CanExecute(object? parameter) => _can?.Invoke(parameter) ?? true;
        public void Execute(object? parameter) => _exec(parameter);
        public event EventHandler? CanExecuteChanged;
        public void RaiseCanExecuteChanged() => CanExecuteChanged?.Invoke(this, EventArgs.Empty);
    }
}
