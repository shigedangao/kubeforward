# Kubeforward >>

Kubeforward is a small interactive CLI which allows to easily expose a Kubernetes pod to your local machine

<p align="center">
    <img align="center" src="./example.gif">
</p>

# Usage

## Default usage

By default, kubeforward will look for the current kubernetes config you're using. It'll also ask the namespace which you want to use

```shell
kubeforward
```

### With a different context

If you wish to use a different kubernetes context you can use the ```-c``` option. The CLI will output a list of kubernetes contexts available in your kubeconfig

```shell
kubeforward -c
```

### With a specified namespace

If you already know the namespace where the pod is located you can use the ```-n``` option.

```shell
kubeforward -n
```

### Combinate the options

Of course you can combine the two options like below

```shell
kubeforward -c -n
```
