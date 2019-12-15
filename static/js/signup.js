class Signup extends HTMLElement {
    constructor() {
        super();
        this.attachShadow({ mode: 'open' });

        let passphrase = this.getAttribute('passphrase')

        this.shadowRoot.innerHTML = `
            <hf-form class="logout" text="Ce compte n'existe pas, voulez-vous le créer ?">
                <input minlength="1" placeholder="name" type="text" name="text" class="name" required/>
                <input type="hidden" value="${passphrase}" class="passphrase" />
                <hf-button content="signup" class="submit"></hf-button>
                <br/>
                <hf-button content="try again" class="abort"></hf-button>
            </hf-form>
        `
    }

    connectedCallback() {
        this.shadowRoot.querySelector(".submit").addEventListener('click', this._signup.bind(this));
        this.shadowRoot.querySelector(".abort").addEventListener('click', this._abort.bind(this));
    }

    disconnectedCallback() {
        this.shadowRoot.querySelector(".submit").removeEventListener('click', this._signup);
        this.shadowRoot.querySelector(".abort").removeEventListener('click', this._abort);
    }

    _abort(e){
        document.dispatchEvent(new CustomEvent('_auth_abort'));
    }

    _signup(e) {
        e.preventDefault();

        let invalid =  this.shadowRoot.querySelector(":invalid");

        if( invalid){
            return;
        }


        console.debug(this.shadowRoot.querySelector(".name").value)

        fetch("http://localhost:8000/signup", {
                method : "POST",
                headers: {
                   cache : "no-cache"
                },
                body : JSON.stringify({
                    name: this.shadowRoot.querySelector(".name").value,
                    passphrase: this.shadowRoot.querySelector(".passphrase").value
                }),
            })
            .then(res => res.text()) // parse response as JSON with res.json
            .then(response => {
                    var event = new CustomEvent('_auth_login', { 'detail': response });
                    document.dispatchEvent(event);
             })
            .catch(err => {
                console.log({service:"auth", status:"KO", error:err})
                alert("sorry, cannot signup")
            });
    }
}
customElements.define('hf-auth-signup', Signup);